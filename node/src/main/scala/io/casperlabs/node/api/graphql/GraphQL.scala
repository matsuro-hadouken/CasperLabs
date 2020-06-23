package io.casperlabs.node.api.graphql

import cats.effect._
import cats.effect.concurrent.Deferred
import cats.effect.implicits._
import cats.implicits._
import fs2.concurrent.Queue
import fs2.{Pipe, Stream}
import io.casperlabs.casper.MultiParentCasperRef.MultiParentCasperRef
import io.casperlabs.catscontrib.{Fs2Compiler, MonadThrowable}
import io.casperlabs.metrics.Metrics
import io.casperlabs.node.api.graphql.GraphQLQuery._
import io.casperlabs.node.api.graphql.ProtocolState.Subscriptions
import io.casperlabs.node.api.graphql.circe._
import io.casperlabs.node.api.graphql.schema.GraphQLSchemaBuilder
import io.casperlabs.shared.{Log, LogSource}
import io.casperlabs.smartcontracts.ExecutionEngineService
import io.casperlabs.storage.block._
import io.casperlabs.storage.deploy.DeployStorage
import io.casperlabs.storage.dag.DagStorage
import io.circe.parser.parse
import io.circe.syntax._
import io.circe.{Json, JsonObject}
import org.http4s.circe._
import org.http4s.server.websocket.WebSocketBuilder
import org.http4s.websocket.WebSocketFrame
import org.http4s.{Header, Headers, HttpRoutes, MalformedMessageBodyFailure, Response, StaticFile}
import sangria.execution._
import sangria.parser.QueryParser
import scala.util.control.NonFatal
import scala.concurrent.ExecutionContext
import scala.concurrent.duration._
import scala.util.{Failure, Success}

/**
  * Entry point of the package.
  */
object GraphQL {

  private[graphql] implicit val MetricsSource = Metrics.Source(Metrics.BaseSource, "graphql")

  private[graphql] val requiredHeaders =
    Headers.of(Header("Upgrade", "websocket"), Header("Sec-WebSocket-Protocol", "graphql-ws"))

  /* Entry point */
  def service[F[_]: ConcurrentEffect: ContextShift: Timer: Log: MultiParentCasperRef: BlockStorage: FinalizedBlocksStream: ExecutionEngineService: DeployStorage: DagStorage: Fs2Compiler: Metrics](
      executionContext: ExecutionContext
  ): HttpRoutes[F] = {
    import io.casperlabs.node.api.graphql.RunToFuture.fromEffect
    implicit val ec: ExecutionContext                            = executionContext
    implicit val fs2SubscriptionStream: Fs2SubscriptionStream[F] = new Fs2SubscriptionStream[F]()
    val schemaBuilder                                            = new GraphQLSchemaBuilder[F]
    buildRoute(
      executor = Executor(
        schemaBuilder.createSchema,
        deferredResolver = schemaBuilder.createDeferredResolver
      ),
      keepAlivePeriod = 10.seconds,
      ec
    )
  }

  private[graphql] def buildRoute[F[_]: Concurrent: ContextShift: Timer: Log: Fs2SubscriptionStream: Metrics](
      executor: Executor[Unit, Unit],
      keepAlivePeriod: FiniteDuration,
      ec: ExecutionContext
  ): HttpRoutes[F] = {
    val dsl = org.http4s.dsl.Http4sDsl[F]
    import dsl._
    HttpRoutes.of[F] {
      case req @ GET -> Root if requiredHeaders.forall(h => req.headers.exists(_ === h)) =>
        handleWebSocket(executor, keepAlivePeriod)
      case GET -> Root =>
        StaticFile
          .fromResource[F]("/graphql-playground.html", Blocker.liftExecutionContext(ec))
          .getOrElseF(NotFound())
      case req @ POST -> Root =>
        val res: F[Response[F]] = for {
          json                 <- req.as[Json]
          query                <- Sync[F].fromEither(json.as[GraphQLQuery])
          isIntrospectionQuery = query.query.contains("__schema")
          runQuery             = processHttpQuery(query, executor, ec).flatMap(Ok(_))
          res <- if (isIntrospectionQuery) {
                  runQuery
                } else {
                  Log[F].debug(s"GraphQL query: ${query.query}") >>
                    Metrics[F].timer("query")(runQuery)
                }
        } yield res

        res.recoverWith {
          case e: ErrorWithResolver           => BadRequest(e.resolveError)
          case e: MalformedMessageBodyFailure => BadRequest(toJson(e))
          case NonFatal(ex) =>
            Log[F].error(s"Error serving GraphQL: $ex") *>
              InternalServerError(toJson(ex))
        }
    }
  }

  private def handleWebSocket[F[_]: Concurrent: Timer: Log: Fs2SubscriptionStream](
      executor: Executor[Unit, Unit],
      keepAlivePeriod: FiniteDuration
  ): F[Response[F]] = {

    def out(
        queue: Queue[F, GraphQLWebSocketMessage],
        stopSignal: F[Unit]
    ): Stream[F, WebSocketFrame] = {
      val keepAlive = Stream
        .awakeEvery[F](keepAlivePeriod)
        .map(
          _ =>
            WebSocketFrame.Text(
              (GraphQLWebSocketMessage.ConnectionKeepAlive: GraphQLWebSocketMessage).asJson
                .toString()
            )
        )
      val output = queue.dequeue
        .map { m =>
          WebSocketFrame.Text(m.asJson.toString())
        }
        .interruptWhen(stopSignal.map(_.asRight[Throwable]))

      output.mergeHaltL(keepAlive)
    }

    for {
      stopSignal <- Deferred[F, Unit]
      queue      <- Queue.bounded[F, GraphQLWebSocketMessage](100)
      response <- WebSocketBuilder[F].build(
                   send = out(queue, stopSignal.get),
                   receive = webSocketProtocolLogic(queue, stopSignal, executor),
                   headers = Headers.of(Header("Sec-WebSocket-Protocol", "graphql-ws"))
                 )
    } yield response
  }

  private[graphql] def webSocketProtocolLogic[F[_]: Concurrent: Log](
      queue: Queue[F, GraphQLWebSocketMessage],
      stopSignal: Deferred[F, Unit],
      executor: Executor[Unit, Unit]
  )(implicit fs2SubscriptionStream: Fs2SubscriptionStream[F]): Pipe[F, WebSocketFrame, Unit] =
    _.interruptWhen(stopSignal.get.map(_.asRight[Throwable]))
      .flatMap {
        case WebSocketFrame.Text(raw, _) =>
          parse(raw)
            .flatMap(_.as[GraphQLWebSocketMessage])
            .fold(
              e => {
                val error =
                  s"Failed to parse GraphQL WebSocket message: $raw, reason: ${e.getMessage}"
                Stream
                  .eval(
                    Log[F].warn(s"${error -> "error" -> null}") >> queue
                      .enqueue1(GraphQLWebSocketMessage.ConnectionError(error))
                  )
                  .flatMap(_ => Stream.empty.covary[F])
              },
              m =>
                Stream.eval[F, GraphQLWebSocketMessage](
                  Log[F].debug(s"Received subscription query: $m") >> m.pure[F]
                )
            )
        case _ => Stream.empty.covary[F]
      }
      .evalMapAccumulate[F, ProtocolState, Unit](ProtocolState.WaitingForInit) {
        case (ProtocolState.WaitingForInit, GraphQLWebSocketMessage.ConnectionInit) =>
          for {
            _ <- queue.enqueue1(GraphQLWebSocketMessage.ConnectionAck)
            _ <- queue.enqueue1(GraphQLWebSocketMessage.ConnectionKeepAlive)
          } yield (ProtocolState.Active[F](Map.empty), ())

        case (
            ProtocolState.Active(activeSubscriptions),
            GraphQLWebSocketMessage.Start(id, payload)
            ) =>
          for {
            _ <- activeSubscriptions
                  .asInstanceOf[Subscriptions[F]]
                  .get(id)
                  .fold(().pure[F]) { prevFiber =>
                    prevFiber.cancel
                  }
            fiber <- processWebSocketQuery[F](payload, executor)
                      .map(json => GraphQLWebSocketMessage.Data(id, json))
                      .onFinalizeCase {
                        case ExitCase.Error(e) =>
                          queue.enqueue1(GraphQLWebSocketMessage.Error(id, e.getMessage))
                        case ExitCase.Completed =>
                          queue.enqueue1(GraphQLWebSocketMessage.Complete(id))
                        case ExitCase.Canceled => ().pure[F]
                      }
                      .evalMap(queue.enqueue1)
                      .compile
                      .toList
                      .void
                      .start
          } yield (
            ProtocolState.Active[F](
              activeSubscriptions.asInstanceOf[Subscriptions[F]] + (id -> fiber)
            ),
            ()
          )

        case (ProtocolState.Active(activeSubscriptions), GraphQLWebSocketMessage.Stop(id)) =>
          for {
            _ <- activeSubscriptions
                  .asInstanceOf[Subscriptions[F]]
                  .get(id)
                  .fold(().pure[F])(_.cancel)
          } yield (
            ProtocolState.Active(activeSubscriptions.asInstanceOf[Subscriptions[F]] - id),
            ()
          )

        case (
            ProtocolState.Active(activeSubscriptions),
            GraphQLWebSocketMessage.ConnectionTerminate
            ) =>
          for {
            _ <- activeSubscriptions
                  .asInstanceOf[Subscriptions[F]]
                  .values
                  .toList
                  .traverse(_.cancel)
            _ <- stopSignal.complete(())
          } yield (ProtocolState.Closed, ())

        case (_, GraphQLWebSocketMessage.ConnectionTerminate) =>
          for {
            _ <- stopSignal.complete(())
          } yield (ProtocolState.Closed, ())

        case (protocolState, message) =>
          val error = s"Unexpected message: $message in state: '${protocolState.name}', ignoring"
          for {
            _ <- Log[F].warn(s"${error -> "error" -> null}")
            _ <- queue.enqueue1(GraphQLWebSocketMessage.ConnectionError(error))
          } yield (protocolState, ())
      }
      .drain

  private def processWebSocketQuery[F[_]: MonadThrowable](
      query: GraphQLQuery,
      executor: Executor[Unit, Unit]
  )(
      implicit fs2SubscriptionStream: Fs2SubscriptionStream[F]
  ): Stream[F, Json] = {
    import sangria.execution.ExecutionScheme.Stream
    fs2.Stream
      .fromEither[F](QueryParser.parse(query.query).toEither)
      .flatMap(queryAst => executor.execute(queryAst, (), ()))
  }

  private def processHttpQuery[F[_]: Async](
      query: GraphQLQuery,
      executor: Executor[Unit, Unit],
      ec: ExecutionContext
  ): F[Json] =
    Async[F]
      .fromTry(QueryParser.parse(query.query))
      .flatMap { queryAst =>
        Async[F].async[Json](
          callback =>
            executor
              .execute[Json](queryAst, (), (), none[String], Json.fromJsonObject(JsonObject.empty))
              .onComplete {
                case Success(json) =>
                  callback(json.asRight[Throwable])
                case Failure(e) =>
                  callback(e.asLeft[Json])
              }(ec)
        )
      }

  private def toJson(e: Throwable): Json =
    Json.fromJsonObject(
      JsonObject(
        "errors" -> Json.fromValues(
          List(
            Json.fromJsonObject(
              JsonObject(
                "message" -> Json.fromString(e.getMessage)
              )
            )
          )
        )
      )
    )
}

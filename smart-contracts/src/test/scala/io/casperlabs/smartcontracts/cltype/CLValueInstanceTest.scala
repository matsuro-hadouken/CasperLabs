package io.casperlabs.smartcontracts.cltype

import cats.Functor
import cats.implicits._
import eu.timepit.refined._
import eu.timepit.refined.api.Refined
import eu.timepit.refined.numeric._
import io.casperlabs.smartcontracts.bytesrepr.ToBytes
import org.scalatest.{FlatSpec, Matchers}
import org.scalatest.prop.PropertyChecks
import scala.annotation.tailrec
import CLTypeSerializationTest.nested
import KeySerializationTest.arbKey
import URefSerializationTest.arbURef

class CLValueInstanceTest extends FlatSpec with Matchers with PropertyChecks {
  "CLValue instantiation" should "be stack safe (deep nesting)" in {
    val n        = 1000000
    val deepType = nested(CLType.Bool, n)(t => CLType.Option(t))
    val bytes    = Array.fill[Byte](n + 1)(1)
    val instance = CLValueInstance.from(CLValue(deepType, bytes))

    @tailrec
    def checkInstance(i: CLValueInstance, count: Int = 0): Unit =
      if (count == n) {
        i shouldBe CLValueInstance.Bool(true)
      } else {
        checkInstance(i.asInstanceOf[CLValueInstance.Option].value.get, count + 1)
      }

    checkInstance(instance.right.get)
  }

  it should "be stack safe (wide nesting)" in {
    val n        = 20
    val k        = math.pow(2, n.toDouble).toInt
    val wideType = nested(CLType.Bool, n)(t => CLType.Tuple2(t, t))
    val bytes    = Array.fill[Byte](k)(1)
    val instance = CLValueInstance.from(CLValue(wideType, bytes))

    @tailrec
    def checkInstance(nodes: Vector[CLValueInstance], depth: Int = 0): Unit =
      if (depth == n) {
        nodes.foreach { n =>
          n shouldBe CLValueInstance.Bool(true)
        }
      } else {
        val nextLevel = nodes.flatMap { n =>
          val t = n.asInstanceOf[CLValueInstance.Tuple2]
          Vector(t._1, t._2)
        }
        checkInstance(nextLevel, depth + 1)
      }

    checkInstance(Vector(instance.right.get))
  }

  it should "instantiate CLType.Bool properly" in forAll { (b: Boolean) =>
    instantiateTest(b, CLType.Bool, CLValueInstance.Bool.apply)
  }

  it should "instantiate CLType.I32 properly" in forAll { (i: Int) =>
    instantiateTest(i, CLType.I32, CLValueInstance.I32.apply)
  }

  it should "instantiate CLType.I64 properly" in forAll { (i: Long) =>
    instantiateTest(i, CLType.I64, CLValueInstance.I64.apply)
  }

  it should "instantiate CLType.U8 properly" in forAll { (i: Byte) =>
    instantiateTest(i, CLType.U8, CLValueInstance.U8.apply)
  }

  it should "instantiate CLType.U32 properly" in forAll { (i: Int) =>
    instantiateTest(i, CLType.U32, CLValueInstance.U32.apply)
  }

  it should "instantiate CLType.U64 properly" in forAll { (i: Long) =>
    instantiateTest(i, CLType.U64, CLValueInstance.U64.apply)
  }

  it should "instantiate CLType.U128 properly" in forAll { (i: BigInt) =>
    whenever(i >= 0) {
      instantiateTest[BigInt](
        i,
        CLType.U128,
        x => CLValueInstance.U128(refineV[NonNegative](x).right.get)
      )
    }
  }

  it should "instantiate CLType.U256 properly" in forAll { (i: BigInt) =>
    whenever(i >= 0) {
      instantiateTest[BigInt](
        i,
        CLType.U128,
        x => CLValueInstance.U128(refineV[NonNegative](x).right.get)
      )
    }
  }

  it should "instantiate CLType.U512 properly" in forAll { (i: BigInt) =>
    whenever(i >= 0) {
      instantiateTest[BigInt](
        i,
        CLType.U128,
        x => CLValueInstance.U128(refineV[NonNegative](x).right.get)
      )
    }
  }

  it should "instantiate CLType.Unit properly" in {
    instantiateTest[Unit]((), CLType.Unit, _ => CLValueInstance.Unit)
  }

  it should "instantiate CLType.String properly" in forAll { (s: String) =>
    instantiateTest(s, CLType.String, CLValueInstance.String.apply)
  }

  it should "instantiate CLType.Key properly" in forAll { (k: Key) =>
    instantiateTest(k, CLType.Key, CLValueInstance.Key.apply)
  }

  it should "instantiate CLType.URef properly" in forAll { (u: URef) =>
    instantiateTest(u, CLType.URef, CLValueInstance.URef.apply)
  }

  it should "instantiate CLType.Option properly" in forAll { (o: Option[Long]) =>
    instantiateTestF[Option, Long](
      o,
      CLType.U64,
      CLType.Option.apply,
      CLValueInstance.U64.apply,
      (x, t) => CLValueInstance.Option(x, t).right.get
    )
  }

  it should "instantiate CLType.List properly" in forAll { (list: List[Key]) =>
    instantiateTestF[List, Key](
      list,
      CLType.Key,
      CLType.List.apply,
      CLValueInstance.Key.apply,
      (x, t) => CLValueInstance.List(x, t).right.get
    )
  }

  it should "instantiate CLType.FixedList properly" in forAll { (list: List[String]) =>
    val n = list.size
    instantiateTestF[List, String](
      list,
      CLType.String,
      CLType.FixedList(_, n),
      CLValueInstance.String.apply,
      (x, t) => CLValueInstance.FixedList(x, t, n).right.get
    )(Functor[List], toBytesFixedList[String])
  }

  it should "instantiate CLType.Result properly" in forAll { (e: Either[String, URef]) =>
    instantiateTest[Either[String, URef]](
      e,
      CLType.Result(ok = CLType.URef, err = CLType.String),
      x =>
        CLValueInstance
          .Result(
            x.bimap(CLValueInstance.String.apply, CLValueInstance.URef.apply),
            CLType.URef,
            CLType.String
          )
          .right
          .get
    )
  }

  it should "instantiate CLType.Map properly" in forAll { (m: Map[Int, Boolean]) =>
    instantiateTest[Map[Int, Boolean]](
      m,
      CLType.Map(CLType.I32, CLType.Bool),
      x =>
        CLValueInstance
          .Map(
            x.map { case (k, v) => (CLValueInstance.I32(k), CLValueInstance.Bool(v)) },
            CLType.I32,
            CLType.Bool
          )
          .right
          .get
    )
  }

  it should "instantiate CLType.Tuple1 properly" in forAll { (t: Tuple1[Byte]) =>
    instantiateTest[Tuple1[Byte]](
      t,
      CLType.Tuple1(CLType.U8),
      x => CLValueInstance.Tuple1(CLValueInstance.U8(x._1))
    )
  }

  it should "instantiate CLType.Tuple2 properly" in forAll { (t: (Long, Long)) =>
    instantiateTest[(Long, Long)](
      t,
      CLType.Tuple2(CLType.U64, CLType.I64),
      x => CLValueInstance.Tuple2(CLValueInstance.U64(x._1), CLValueInstance.I64(x._2))
    )
  }

  it should "instantiate CLType.Tuple3 properly" in forAll {
    (t: (String, Option[Int], Seq[Byte])) =>
      instantiateTest[(String, Option[Int], Seq[Byte])](
        t,
        CLType.Tuple3(CLType.String, CLType.Option(CLType.I32), CLType.List(CLType.U8)),
        x =>
          CLValueInstance.Tuple3(
            CLValueInstance.String(x._1),
            CLValueInstance.Option(x._2.map(CLValueInstance.I32.apply), CLType.I32).right.get,
            CLValueInstance.List(x._3.map(CLValueInstance.U8.apply), CLType.U8).right.get
          )
      )
  }

  private def instantiateTest[T: ToBytes](t: T, clType: CLType, instance: T => CLValueInstance) = {
    val clValue    = CLValue.from(t, clType)
    val clInstance = CLValueInstance.from(clValue)

    clInstance shouldBe Right(instance(t))
  }

  private def instantiateTestF[F[_]: Functor, T](
      ft: F[T],
      innerType: CLType,
      fType: CLType => CLType,
      innerInstance: T => CLValueInstance,
      fInstance: (F[CLValueInstance], CLType) => CLValueInstance
  )(implicit toBytes: ToBytes[F[T]]) =
    instantiateTest[F[T]](
      ft,
      fType(innerType),
      x => fInstance(x.map(innerInstance), innerType)
    )

  private implicit def toBytesList[T: ToBytes]: ToBytes[List[T]] = new ToBytes[List[T]] {
    def toBytes(list: List[T]): Array[Byte] = ToBytes.toBytesSeq[T].toBytes(list)
  }

  private def toBytesFixedList[T: ToBytes]: ToBytes[List[T]] = new ToBytes[List[T]] {
    // length is not included in a FixedList
    def toBytes(list: List[T]): Array[Byte] = ToBytes.toBytesSeq[T].toBytes(list).drop(4)
  }
}

package io.casperlabs.smartcontracts.cltype

import io.casperlabs.smartcontracts.bytesrepr.{BytesView, FromBytes, ToBytes}

/** Special type indicating a byte array with exactly 32 elements. */
case class ByteArray32 private (bytes: IndexedSeq[Byte])

object ByteArray32 {
  def apply(bytes: IndexedSeq[Byte]): Option[ByteArray32] =
    if (bytes.length == 32) Some(new ByteArray32(bytes))
    else None

  implicit val toBytes: ToBytes[ByteArray32] = new ToBytes[ByteArray32] {
    override def toBytes(b: ByteArray32): Array[Byte] =
      b.bytes.toArray
  }

  def lt(a: ByteArray32, b: ByteArray32) = (a, b) match {
    case (ByteArray32(xs), ByteArray32(ys)) =>
      xs.iterator.zip(ys.iterator).exists {
        case (x, y) => x < y
      }
  }

  val deserializer: FromBytes.Deserializer[ByteArray32] =
    FromBytes.take(32).map(view => new ByteArray32(view.toArray))
}

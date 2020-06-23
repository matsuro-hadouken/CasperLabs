//@ts-nocheck
import * as CL from "../../../../contract-as/assembly";
import {Error, ErrorCode} from "../../../../contract-as/assembly/error";
import {fromBytesString, toBytesMap} from "../../../../contract-as/assembly/bytesrepr";
import {Key} from "../../../../contract-as/assembly/key";
import {CLValue, CLType, CLTypeTag} from "../../../../contract-as/assembly/clvalue";
import {Pair} from "../../../../contract-as/assembly/pair";

const ENTRY_FUNCTION_NAME = "delegate";
const HASH_KEY_NAME = "do_nothing_hash";
const PACKAGE_HASH_KEY_NAME = "do_nothing_package_hash";
const ACCESS_KEY_NAME = "do_nothing_access";
const CONTRACT_VERSION = "contract_version";

export function delegate(): void {
  // no-op
}

export function call(): void {
  let entryPoints = new CL.EntryPoints();
  let entryPoint = new CL.EntryPoint("delegate", new Array<Pair<String, CLType>>(), new CLType(CLTypeTag.Unit), new CL.PublicAccess(), CL.EntryPointType.Contract);
  entryPoints.addEntryPoint(entryPoint);

  const result = CL.newContract(
    entryPoints,
    null,
    PACKAGE_HASH_KEY_NAME,
    ACCESS_KEY_NAME,
  );
  const key = Key.create(CLValue.fromI32(result.contractVersion));
  if (key === null) {
    return;
  }
  CL.putKey(CONTRACT_VERSION, key);
  CL.putKey(HASH_KEY_NAME, Key.fromHash(result.contractHash));
}

//@ts-nocheck
import * as CL from "../../../../contract-as/assembly";
import {Error, ErrorCode} from "../../../../contract-as/assembly/error";
import {fromBytesString} from "../../../../contract-as/assembly/bytesrepr";
import {Key} from "../../../../contract-as/assembly/key";
import {Pair} from "../../../../contract-as/assembly/pair";
import {URef} from "../../../../contract-as/assembly/uref";
import {Pair} from "../../../../contract-as/assembly/pair";
import {createPurse} from "../../../../contract-as/assembly/purse";
import {CLType, CLTypeTag} from "../../../../contract-as/assembly/clvalue";
import * as CreatePurse01 from "../../create-purse-01/assembly";

const ENTRY_FUNCTION_NAME = "delegate";
const DO_NOTHING_PACKAGE_HASH_KEY_NAME = "do_nothing_package_hash";
const DO_NOTHING_ACCESS_KEY_NAME = "do_nothing_access";

export function delegate(): void {
  let key = new Uint8Array(32);
  for (var i = 0; i < 32; i++) {
    key[i] = 1;
  }
  CL.putKey("called_do_nothing_ver_2", Key.fromHash(key));
  CreatePurse01.delegate();
}

export function call(): void {
  let entryPoints = new CL.EntryPoints();
  let entryPoint = new CL.EntryPoint(
    ENTRY_FUNCTION_NAME,
    new Array<Pair<String, CLType>>(),
    new CLType(CLTypeTag.Unit),
    new CL.PublicAccess(),
    CL.EntryPointType.Session);
  entryPoints.addEntryPoint(entryPoint);

  let doNothingPackageHash = CL.getKey(DO_NOTHING_PACKAGE_HASH_KEY_NAME);
  if (doNothingPackageHash === null) {
    Error.fromErrorCode(ErrorCode.None).revert();
    return;
  }

  let doNothingURef = CL.getKey(DO_NOTHING_ACCESS_KEY_NAME);
  if (doNothingURef === null) {
    Error.fromErrorCode(ErrorCode.None).revert();
    return;
  }

  const result = CL.addContractVersion(
    <Uint8Array>doNothingPackageHash.hash,
    entryPoints,
    new Array<Pair<String, Key>>(),
  );

  CL.putKey("end of upgrade", Key.fromHash(result.contractHash));
}

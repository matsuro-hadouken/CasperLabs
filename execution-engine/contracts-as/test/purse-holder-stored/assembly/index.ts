//@ts-nocheck
import * as CL from "../../../../contract-as/assembly";
import {Error, ErrorCode} from "../../../../contract-as/assembly/error";
import {fromBytesString, toBytesMap} from "../../../../contract-as/assembly/bytesrepr";
import {Key} from "../../../../contract-as/assembly/key";
import {Pair} from "../../../../contract-as/assembly/pair";
import {putKey, ret} from "../../../../contract-as/assembly";
import {CLValue, CLType, CLTypeTag} from "../../../../contract-as/assembly/clvalue";
import {createPurse} from "../../../../contract-as/assembly/purse";
import {URef} from "../../../../contract-as/assembly/uref";
import {CLTypeTag} from "../../../../contract-as/assembly/clvalue";

const METHOD_ADD = "add";
const METHOD_REMOVE = "remove";
const METHOD_VERSION = "version";

const ENTRY_POINT_ADD = "add_named_purse";
const ENTRY_POINT_VERSION = "version";
const HASH_KEY_NAME = "purse_holder";
const ACCESS_KEY_NAME = "purse_holder_access";
const ARG_PURSE = "purse_name";
const VERSION = "1.0.0";
const PURSE_HOLDER_STORED_CONTRACT_NAME = "purse_holder_stored";

enum CustomError {
  MissingMethodNameArg = 0,
  InvalidMethodNameArg = 1,
  MissingPurseNameArg = 2,
  InvalidPurseNameArg = 3,
  UnknownMethodName = 4,
  NamedPurseNotCreated = 5
}

export function add_named_purse(): void {
  const purseNameBytes = CL.getNamedArg(ARG_PURSE);
  const purseName = fromBytesString(purseNameBytes).unwrap();
  const purse = createPurse();
  CL.putKey(purseName, Key.fromURef(purse));
}

export function version(): void {
    CL.ret(CLValue.fromString(VERSION));
}

export function call(): void {
  let entryPoints = new CL.EntryPoints();

  {
    let args = new Array<Pair<String, CLType>>();
    args.push(new Pair(ARG_PURSE, new CLType(CLTypeTag.String)));
    let entryPointAdd = new CL.EntryPoint(ENTRY_POINT_ADD, args, new CLType(CLTypeTag.Unit), new CL.PublicAccess(), CL.EntryPointType.Contract);
    entryPoints.addEntryPoint(entryPointAdd);  
  }
  {
    let entryPointAdd = new CL.EntryPoint(ENTRY_POINT_VERSION, new Array<Pair<String, CLType>>(), new CLType(CLTypeTag.Unit), new CL.PublicAccess(), CL.EntryPointType.Contract);
    entryPoints.addEntryPoint(entryPointAdd);
  }

  let result = CL.newContract(
    entryPoints,
    null,
    HASH_KEY_NAME,
    ACCESS_KEY_NAME);

  putKey(PURSE_HOLDER_STORED_CONTRACT_NAME, Key.fromHash(result.contractHash));
  const versionKey = Key.create(CLValue.fromString(VERSION));
  if (versionKey === null) {
    Error.fromErrorCode(ErrorCode.Formatting).revert();
  }
  putKey(ENTRY_POINT_VERSION, <Key>versionKey);
}

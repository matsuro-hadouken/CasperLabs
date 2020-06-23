//@ts-nocheck
import {getMainPurse} from "../../../../contract-as/assembly/account";
import * as CL from "../../../../contract-as/assembly";
import {Error} from "../../../../contract-as/assembly/error";
import {URef} from "../../../../contract-as/assembly/uref";

const ARG_PURSE = "purse";

enum CustomError {
  MissingExpectedMainPurseArg = 86,
  InvalidExpectedMainPurseArg = 97,
  EqualityAssertionFailed = 139
}

export function call(): void {
  let expectedMainPurseArg = CL.getNamedArg(ARG_PURSE);
  let purseResult = URef.fromBytes(expectedMainPurseArg);
  if (purseResult === null){
    Error.fromUserError(<u16>CustomError.InvalidExpectedMainPurseArg).revert();
    return;
  }
  const expectedMainPurse = purseResult.value;
  const actualMainPurse = getMainPurse();

  if (<URef>expectedMainPurse != <URef>actualMainPurse)
    Error.fromUserError(<u16>CustomError.EqualityAssertionFailed).revert();
}

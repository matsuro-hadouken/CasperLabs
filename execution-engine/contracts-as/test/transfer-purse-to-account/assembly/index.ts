//@ts-nocheck
import * as CL from "../../../../contract-as/assembly";
import {Error, ErrorCode} from "../../../../contract-as/assembly/error";
import {U512} from "../../../../contract-as/assembly/bignum";
import {getMainPurse} from "../../../../contract-as/assembly/account";
import {Key} from "../../../../contract-as/assembly/key";
import {putKey} from "../../../../contract-as/assembly";
import {CLValue} from "../../../../contract-as/assembly/clvalue";
import {getPurseBalance, transferFromPurseToAccount, TransferredTo} from "../../../../contract-as/assembly/purse";
import {URef} from "../../../../contract-as/assembly/uref";


const TRANSFER_RESULT_UREF_NAME = "transfer_result";
const MAIN_PURSE_FINAL_BALANCE_UREF_NAME = "final_balance";

const ARG_TARGET = "target";
const ARG_AMOUNT = "amount";

enum CustomError{
    MissingAmountArg = 1,
    InvalidAmountArg = 2,
    MissingDestinationAccountArg = 3,
    UnableToGetBalance = 103
}

export function delegate(): void {
    const mainPurse = getMainPurse();
    const destinationAccountAddrArg = CL.getNamedArg(ARG_TARGET);
    const amountArg = CL.getNamedArg(ARG_AMOUNT);
    const amountResult = U512.fromBytes(amountArg);
    if (amountResult.hasError()) {
        Error.fromUserError(<u16>CustomError.InvalidAmountArg).revert();
        return;
    }
    let amount = amountResult.value;
    let message = "";
    const result = transferFromPurseToAccount(<URef>mainPurse, <Uint8Array>destinationAccountAddrArg, amount);
    switch (result) {
        case TransferredTo.NewAccount:
            message = "Ok(NewAccount)";
            break;
        case TransferredTo.ExistingAccount:
            message = "Ok(ExistingAccount)";
            break;
        case TransferredTo.TransferError:
            message = "Err(ApiError::Transfer [" + ErrorCode.Transfer.toString() + "])";
            break;
    }
    const transferResultKey = Key.create(CLValue.fromString(message));
    putKey(TRANSFER_RESULT_UREF_NAME, <Key>transferResultKey);
    const maybeBalance = getPurseBalance(mainPurse);
    if (maybeBalance === null) {
        Error.fromUserError(<u16>CustomError.UnableToGetBalance).revert();
        return;
    }
    const key = Key.create(CLValue.fromU512(<U512>maybeBalance));
    putKey(MAIN_PURSE_FINAL_BALANCE_UREF_NAME, <Key>key);
}

export function call(): void {
    delegate();
}

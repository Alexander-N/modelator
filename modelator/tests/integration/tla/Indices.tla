------------------------------ MODULE Indices ------------------------------
\* This is the model of Substrate Frame Indices from 
\* https://github.com/paritytech/substrate/tree/master/frame/indices 
\* at this commit: https://git.io/Ju5Jv
\*
\* An index is a short form of an address.
\* This module handles allocation of indices for a newly created accounts.
\*
\* This TLA+ model is created for demostrative purposes of MBT capabilities.
\* s2021 Andrey Kuprianov, Informal Systems

EXTENDS Integers, FiniteSets

CONSTANTS 
  \* @typeAlias: ADDR = Int;
  \* @typeAlias: INDEX = Int;
  \* @typeAlias: BALANCE = Int;
  \* @typeAlias: INDEX_ENTRY = [ addr: ADDR, deposit: Int, perm: Bool ];
  \* @typeAlias: INDICES = INDEX -> INDEX_ENTRY;
  \* @typeAlias: RESERVES = ADDR -> BALANCE;
  \* @typeAlias: ACTION = [type: Str, who: ADDR, new: ADDR, index: INDEX, freeze: Bool];
  \* How many users and indices are there? 
  \* @type: Int;
  NUM_USERS,
  \* @type: Int;
  NUM_INDICES

VARIABLES
 \* @type: INDICES;
  indices,
  \* @type: RESERVES;
  reserved,
  \* @type: ACTION;
  action,
  \* @type: Str;
  actionOutcome

Indices == 1..NUM_INDICES
Addresses == 0..NUM_USERS

\* A non-address
None == -1
\* A root address
Root == 0


\* Possible outcomes

\* Operation successful
OK == "OK"
\* The index was not already assigned.
NotAssigned == "NotAssigned"
\* The index is assigned to another account.
NotOwner == "NotOwner"
\* Only root can do this.
NotRoot == "NotRoot"
\* The index was not available.
InUse == "InUse"
\* The source and destination accounts are identical.
NotTransfer == "NotTransfer"
\* The index is permanent and may not be freed/changed.
Permanent == "Permanent"


ClaimOutcome(who, index) ==
  IF indices[index].who /= None
    THEN InUse
  ELSE OK

\* @type: (ADDR, INDEX) => Bool;
Claim(who, index) ==
  /\ indices' = [indices EXCEPT ![index] = 
       [who |-> who, deposit |-> 1, perm |-> FALSE] ]
  /\ reserved' = [reserved EXCEPT ![who] = @ + 1]


\* @type: (ADDR, ADDR, INDEX) => Str;
TransferOutcome(who, new, index) ==
  IF who = new
    THEN NotTransfer
  ELSE IF indices[index].who = None
    THEN NotAssigned
  ELSE IF indices[index].perm 
    THEN Permanent
  ELSE IF indices[index].who /= who
    THEN NotOwner
  ELSE 
    OK

\* @type: (ADDR, ADDR, INDEX) => Bool;
Transfer(who, new, index)==
  LET amount == indices[index].deposit IN
  /\ indices' = [indices EXCEPT ![index] = [who |-> new, deposit |-> amount, perm |-> FALSE]]
  /\ reserved' = [reserved EXCEPT ![who] = @ - amount, ![new] = @ + amount]


\* @type: (ADDR, INDEX) => Str;
FreeOutcome(who, index) ==
  IF indices[index].who = None
    THEN NotAssigned
  ELSE IF indices[index].perm 
    THEN Permanent
  ELSE IF indices[index].who /= who
    THEN NotOwner
  ELSE 
    OK

\* @type: (ADDR, INDEX) => Bool;
Free(who, index)==
  /\ indices' = [indices EXCEPT ![index] = [who |-> None, deposit |-> 0, perm |-> FALSE]]
  /\ reserved' = [reserved EXCEPT ![who] = @ - indices[index].deposit]


\* @type: (ADDR, ADDR, INDEX, Bool) => Str;
ForceTransferOutcome(who, new, index, freeze) ==
  IF who /= Root
    THEN NotRoot
  ELSE 
    OK

\* @type: (ADDR, ADDR, INDEX, Bool) => Bool;
ForceTransfer(who, new, index, freeze)==
  LET entry == indices[index] IN
    /\ indices' = [indices EXCEPT ![index] = [who |-> new, deposit |-> 0, perm |-> freeze]]
    /\ IF entry.who /= None THEN
         reserved' = [reserved EXCEPT ![who] = @ - indices[index].deposit]
       ELSE
         UNCHANGED reserved

\* @type: (ADDR, INDEX) => Str;
FreezeOutcome(who, index) ==
  IF indices[index].who = None
    THEN NotAssigned
  ELSE IF indices[index].perm
    THEN Permanent
  ELSE IF indices[index].who /= who
    THEN NotOwner
  ELSE OK

\* @type: (ADDR, INDEX) => Bool;
Freeze(who, index) ==
  /\ indices' = [indices EXCEPT ![index] = [who |-> who, deposit |-> 0, perm |-> TRUE]]
  /\ reserved' = [reserved EXCEPT ![who] = @ - indices[index].deposit]


\* Start with no occupied indices, and no reserved coins
Init == 
  /\ indices = [ i \in Indices |-> [ who |-> None, deposit |-> 0, perm |-> FALSE ] ]
  /\ reserved = [ a \in Addresses |-> 0 ]
  /\ action = [ type |-> "None" ]
  /\ actionOutcome = "None"


=============================================================================



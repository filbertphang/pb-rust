import LeanSts.State
import LeanSts.BFT.Network
import ReliableBroadcast

-- lean-rust interfacing
-- ===
-- we construct concrete versions of the protocol in lean so that we do not need to pass type parameters
-- between Lean and Rust.

-- concrete address, round, and value parameters
-- (using "abbrev" instead of "def" for reducibility, i want ConcreteValue to have the properties of String.)
abbrev ConcreteAddress := String
abbrev ConcreteRound := USize
abbrev ConcreteValue := String

-- concrete instantiations of the RB
abbrev ConcreteRBMessage := (@Message ConcreteAddress ConcreteRound ConcreteValue)
abbrev ConcreteRBState := (@NodeState ConcreteAddress ConcreteRound ConcreteValue)
abbrev ConcreteRBPacket := (Packet ConcreteAddress ConcreteRBMessage)
abbrev ConcreteRBInternalTransition := @InternalTransition ConcreteRound
abbrev ConcreteRBProtocol := @NetworkProtocol ConcreteAddress (ConcreteRBMessage) (ConcreteRBState) (ConcreteRBInternalTransition)

-- function that calls rust to determine what the leader node's message payload is.
-- we expect this to always be called "get_node_value"
@[extern "get_node_value"]
opaque get_node_value : ConcreteAddress → ConcreteValue

-- function that creates a protocol in lean
-- rust expects this to always be called "create_protocol"
@[export create_protocol]
def create_protocol (node_arr: Array ConcreteAddress) : ConcreteRBProtocol :=
  let node_list := Array.toList node_arr
  -- note: we still have to hard-code String.decEq and USize.decEq here.
  -- would be good if we could derive DecideableEq on our abbrevs, so we can use decEq from the abbrev type directly.
  @RBProtocol ConcreteAddress ConcreteRound ConcreteValue String.decEq USize.decEq String.decEq (node_list) (get_node_value)

@[export create_message]
def create_message (tag: USize) (originator: ConcreteAddress) (r: ConcreteRound) (v: ConcreteValue)  : ConcreteRBMessage :=
  -- for some reason, i can't seem to match on USize directly,
  -- so we cast the tag to Nat first.
  -- this just saves us the trouble of constructing the lean object in rust.
  let tag' := USize.toNat tag;
  match tag' with
  | 0 => Message.InitialMsg r v
  | 1 => Message.EchoMsg originator r v
  | 2 => Message.VoteMsg originator r v
  | _ => sorry

@[export create_packet]
def create_packet (src: ConcreteAddress) (dst: ConcreteAddress) (msg: ConcreteRBMessage) (consumed: Bool) : ConcreteRBPacket :=
  {src := src, dst := dst, msg := msg, consumed := consumed}

@[export init_node_state]
def init_node_state (p: ConcreteRBProtocol) (node_address: ConcreteAddress) : ConcreteRBState :=
  p.localInit node_address

@[export send_message]
def send_message (p: ConcreteRBProtocol) (node_state: ConcreteRBState) (round: ConcreteRound) : ConcreteRBState × Array ConcreteRBPacket :=
  let (new_state, packet_list) := p.procInternal node_state round
  (new_state, List.toArray packet_list)

@[export handle_message]
def handle_message (p: ConcreteRBProtocol) (node_state: ConcreteRBState) (src: ConcreteAddress) (msg: ConcreteRBMessage) : ConcreteRBState × Array ConcreteRBPacket :=
  let (new_state, packet_list) := p.procMessage node_state src msg
  (new_state, List.toArray packet_list)

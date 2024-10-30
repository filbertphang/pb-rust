import LeanSts.State
import LeanSts.BFT.Network
import ReliableBroadcast

-- lean-rust interfacing
-- ===
-- Address -> String
-- Round -> Nat
-- Value -> String (for now)

notation "ConcreteRBProtocol" => @NetworkProtocol String (@Message String Nat String) (@NodeState String Nat String) (@InternalTransition Nat)
notation "ConcreteRBMessage" => (@Message String Nat String)
notation "ConcreteRBState" => (@NodeState String Nat String)
notation "ConcreteRBPacket" => (Packet String ConcreteRBMessage)


-- function that calls rust to determine what the leader node's message payload is.
-- we expect this to always be called "get_node_value"
@[extern "get_node_value"]
opaque get_node_value : String → String

-- function that creates a protocol in lean
-- rust expects this to always be called "create_protocol"
@[export create_protocol]
def create_protocol (node_arr: Array String) : ConcreteRBProtocol :=
  let node_list := Array.toList node_arr
  @RBProtocol String Nat String String.decEq Nat.decEq String.decEq (node_list) (get_node_value)

@[export init_node_state]
def init_node_state (p: ConcreteRBProtocol) (node_address: String) : ConcreteRBState :=
  p.localInit node_address

@[export send_message]
def send_message (p: ConcreteRBProtocol) (node_state: ConcreteRBState) (round: Nat) : ConcreteRBState × Array ConcreteRBPacket :=
  let (new_state, packet_list) := p.procInternal node_state round
  (new_state, List.toArray packet_list)

@[export handle_message]
def handle_message (p: ConcreteRBProtocol) (node_state: ConcreteRBState) (src: String) (msg: ConcreteRBMessage) : ConcreteRBState × Array ConcreteRBPacket :=
  let (new_state, packet_list) := p.procMessage node_state src msg
  (new_state, List.toArray packet_list)

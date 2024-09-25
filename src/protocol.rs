use std::collections::{HashMap, HashSet};

type Address = i32;
type Round = i32;

//  partial signature is the same as LightSignature in the actual code.
type PartialSignature = ();
type CombinedSignature = ();

// this is a proof that a Value is sent by the sender.
// specifically, we can use the external validy function EV(value, proof, sender)
// to check that <proof> is a valid proof that <value> was sent by <sender>.
// (note: original defn on bythos paper doesn't include sender, but assumes
// sender's public key is known. this is probably because they use address as PK.)
type Proof = ();

pub enum InternalEvent {
    SendAction { round: Round },
}

#[derive(Clone)]
pub enum Message<V> {
    Init {
        round: Round,
        value: V,
        proof: Proof,
    },
    Echo {
        round: Round,
        // PartialSignature (from the paper) or LightSignature (from the source code)?
        partial_signature: PartialSignature,
    },
}

pub struct Packet<V> {
    src: Address,
    dst: Address,
    msg: Message<V>,
    received: bool,
}

pub struct NodeState<V> {
    address: Address,
    // sender state
    // if Round in Set, then this node is the sender for that round.
    sent: HashSet<Round>,
    // TODO: might want to use a second hash map for this (HashMap<Address, PartialSignature>)
    counter: HashMap<Round, Vec<(Address, PartialSignature)>>,
    output: HashMap<Round, CombinedSignature>,
    // receiver state
    // (Address, Round) means we echoed a message from Address at Round.
    echoed: HashMap<(Address, Round), (V, Proof)>,
}

// aside: there are 2 possible designs for the PB interface here.
// 1) (current impl) implement PB parameters as a struct, and implement PB functions as an impl of that struct
// 2) implement PB parameters as a trait, and implement PB functions as normal functions that take in some
//    `ProvableBroadcast<V>` instead of `self`.
//    note that this impl is slightly awkward because you can't have variables (like `node_addresses` or
//    `num_byzantine`) in traits, so we would need to store those as getter functions.`
// i haven't referenced the API design in other protocol crates yet, but i'll take a look soon.

// <V> refers to the type of the Value that we want to reach a consensus on.
// TODO: might want to parameterize over Proof type too.
// (as well as other type parameters like PartialSignature, CombinedSignature, etc)
pub struct ProvableBroadcast<V> {
    node_addresses: Vec<Address>,
    num_byzantine: usize,
    // generates some form of value and proof
    value_bft: fn(&Address, &Round) -> (V, Proof),
    // validate a partial signature
    externally_validate: fn(&Round, &V, &Proof) -> bool,
    // TODO: need to add in a parameter for the private key, probably.
    partially_sign: fn(&Address, &Round, &V) -> PartialSignature,
    partially_validate: fn(&Address, &Round, &V, &PartialSignature) -> bool,
    combine_partial_signatures: fn(Vec<&PartialSignature>) -> CombinedSignature,
}

impl<V: Clone> ProvableBroadcast<V> {
    fn signature_threshold(&self) -> usize {
        self.node_addresses.len() - self.num_byzantine
    }

    fn make_packet(src: &Address, dst: &Address, msg: &Message<V>) -> Packet<V> {
        Packet {
            src: src.clone(),
            dst: dst.clone(),
            msg: msg.clone(),
            received: false,
        }
    }
    // broadcasts a message to all nodes.
    // filbs: for simplicity, will just clone everything first.
    fn broadcast(&self, src: &Address, msg: Message<V>) -> Vec<Packet<V>> {
        self.node_addresses
            .iter()
            .map(|dst| ProvableBroadcast::<V>::make_packet(src, dst, &msg))
            .collect()
    }

    // process internal events.
    // this function marks a node as the sender of a given message, and makes it
    // send the Init message to other nodes.
    // (i think the Coq implementation returns NodeState because its functional.
    // but since rust isn't, we can just update the hashmaps/sets directly.)
    fn proc_int(&self, st: &mut NodeState<V>, internal_event: InternalEvent) -> Vec<Packet<V>> {
        let NodeState {
            address: id,
            sent,
            counter,
            output,
            echoed,
        } = st;
        match internal_event {
            InternalEvent::SendAction { round: r } => match sent.get(&r) {
                // this node has already initiated this broadcast.
                Some(_) => {
                    // no further packets need to be sent.
                    Vec::new()
                }
                // this node has not yet initiated this broadcast.
                None => {
                    // mark this node as the sender for this round
                    sent.insert(r);
                    let (v, p) = (self.value_bft)(id, &r);
                    let init_msg = Message::Init {
                        round: r,
                        value: v,
                        proof: p,
                    };
                    self.broadcast(id, init_msg)
                }
            },
        }
    }

    fn proc_msg(&self, st: &mut NodeState<V>, src: Address, msg: Message<V>) -> Vec<Packet<V>> {
        let NodeState {
            address,
            sent,
            counter,
            output,
            echoed,
        } = st;
        match msg {
            Message::Init {
                round,
                value,
                proof,
            } => match (sent.get(address), echoed.get(&(*address, round))) {
                (None, None) => {
                    // we are not the sender, AND we haven't echoed this message before.
                    // so, we should echo this message.

                    // validate sender's signature
                    match (self.externally_validate)(&round, &value, &proof) {
                        true => {
                            // generate partial signature
                            let partial_signature = (self.partially_sign)(&address, &round, &value);

                            // update `echoed` map
                            echoed.insert((src, round), (value, proof));

                            // construct message and convert to packets
                            let msg = Message::Echo::<V> {
                                round,
                                partial_signature,
                            };
                            let packet = ProvableBroadcast::<V>::make_packet(&address, &src, &msg);
                            vec![packet]
                        }
                        // could not validate sender's signature: don't echo.
                        false => Vec::new(),
                    }
                }
                _ => {
                    // do not echo this message
                    Vec::new()
                }
            },
            // echo message: make a combined signature with this message, if you're the sender
            // and you haven't aready updated the counter or input
            Message::Echo {
                round,
                partial_signature,
            } => match (sent.get(address), output.get(&round)) {
                // is the sender AND we haven't made a partial signature yet.
                (Some(_), None) => {
                    let (value, _) = (self.value_bft)(&address, &round);
                    match (self.partially_validate)(&address, &round, &value, &partial_signature) {
                        true => {
                            let partial_signatures = counter.entry(round).or_insert(Vec::new());
                            match partial_signatures.contains(&(src, partial_signature)) {
                                false => {
                                    partial_signatures.push((src, partial_signature));

                                    // (this section is an inlined version of `routine_check`.)
                                    // check if we should combine signatures
                                    let has_exactly_enough_signatures =
                                        partial_signatures.len() == self.signature_threshold();
                                    let has_combined_signature = output.contains_key(&round);

                                    // combine signatures, if possible
                                    if has_exactly_enough_signatures && !has_combined_signature {
                                        let partial_signatures_only =
                                            partial_signatures.iter().map(|(_, x)| x).collect();
                                        let combined_signature = (self.combine_partial_signatures)(
                                            partial_signatures_only,
                                        );
                                        output.insert(round, combined_signature);
                                    }

                                    // no packets to output
                                    Vec::new()
                                }

                                // we've already recorded this signature. no further action needed
                                true => Vec::new(),
                            }
                        }
                        // validation failed: don't update signatures.
                        false => Vec::new(),
                    }
                }

                // either NOT the sender, or is the sender but we've already created the partial signature.
                // do nothing.
                _ => Vec::new(),
            },
        }
    }
}

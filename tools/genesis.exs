# Writes chainspecs/tiny.json: the tiny polkajam chainspec that Jamixir ships (JIP-5 dev
# validators) without its Bootstrap service. Service 0 keeps only the null authorizer (CC0) that every core's
# pool and queue already reference, and each sample service is added with its code preimage and
# lookup entry [0].
# Runs inside a Jamixir checkout, which provides the state encoding:
#   cd jamixir && MIX_ENV=test mix run ../jam-sample-services/tools/genesis.exs
alias Jamixir.ChainSpec
alias StateTrie.Transaction
alias System.State.ServiceAccount
import Codec.Encoder

repo = Path.expand("..", __DIR__)
blob = &File.read!(Path.join([repo, "blobs", &1]))
authorizer = blob.("null_authorizer.pvm")
services = [{7, blob.("hash_chain.jam")}]

{:ok, chainspec} = ChainSpec.from_file("priv/polkajam_chainspec.json")
{:ok, serialized} = ChainSpec.get_serialized_state(chainspec)
tx = Transaction.new(StateTrie.StateTrie.load_state(serialized))

# Formula (9.8) v0.7.2: one lookup entry of z octets counts 2 items and 81 + z octets
holding = fn code ->
  %ServiceAccount{
    code_hash: h(code),
    balance: 0xFFFF_FFFF_FFFF_FFFF,
    gas_limit_g: 10,
    gas_limit_m: 10,
    items_in_storage: 2,
    octets_in_storage: 81 + byte_size(code)
  }
end

install = fn tx, service, code ->
  tx
  |> Transaction.put_service_account(service, holding.(code))
  |> Transaction.put_storage_p(service, h(code), code)
  |> Transaction.put_storage_l(service, h(code), byte_size(code), [0])
end

bootstrap = Transaction.service(tx, 0).code_hash
{:ok, code} = Transaction.fetch_storage_p(tx, 0, bootstrap)

tx =
  tx
  |> Transaction.delete_storage_p(0, bootstrap)
  |> Transaction.delete_storage_l(0, bootstrap, byte_size(code))
  |> install.(0, authorizer)

tx = Enum.reduce(services, tx, fn {service, code}, tx -> install.(tx, service, code) end)

%{data: data} = Transaction.to_serialized_state(Transaction.commit(tx))

genesis_state =
  Map.new(data, fn {k, v} -> {Base.encode16(k, case: :lower), Base.encode16(v, case: :lower)} end)

:ok =
  ChainSpec.to_file(
    %{chainspec | genesis_state: genesis_state},
    Path.join([repo, "chainspecs", "tiny.json"])
  )

IO.puts("#{map_size(data)} state entries")

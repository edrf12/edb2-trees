# Árvores AVL e Rubro-Negras

Implementações em Rust de árvores AVL e árvores rubro-negras, com um visualizador local no navegador usando WebAssembly e benchmarks com Criterion.

[Read in English](README.md).

## Conteúdo do projeto

- `src/tree/avl`: implementação da árvore AVL.
- `src/tree/rbtree`: implementação da árvore rubro-negra inclinada à esquerda.
- `src/wasm.rs`: interface WebAssembly usada pelo app no navegador.
- `web/`: visualizador estático usando o código wasm.
- `benches/tree.rs`: benchmarks Criterion de inserção, remoção e busca.

O visualizador aceita chaves inteiras, exibe metadados da árvore e permite
acompanhar buscas, rotações e mudanças de cor quadro a quadro. O estado da
árvore fica somente na memória do navegador.

## Requisitos

- Rust e Cargo.
- Para o app web: `just`, `wasm-pack`, `miniserve` e o target Rust
  `wasm32-unknown-unknown`.

## Compilar e verificar

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Compile o pacote WebAssembly do visualizador:

```sh
wasm-pack build --target web --out-dir web/pkg --no-pack
```

## Executar o app web localmente

Instale o `just` e o servidor de arquivos estáticos Rust:

```sh
cargo install --locked just
cargo install --locked miniserve
```
O recipe `just dev-web` compila o pacote WASM e serve a página em
<http://localhost:8000>:

```sh
just dev-web
```

## Visualizador web

O visualizador é um app web estático. As operações da árvore são executadas no
módulo Rust compilado para WebAssembly; a página usa HTML, CSS e JavaScript
puros para exibir os snapshots e as etapas das operações. Escolha AVL ou
rubro-negra e insira, remova ou busque chaves inteiras. Pressione Enter no campo
de chave para inserir. Use a linha do tempo, os botões de seta ou os atalhos de
teclado para revisar os quadros. Se uma operação começar durante a reprodução,
ela continua avançando pelos novos quadros. Os quadros de rotação e recoloração
mostram snapshots alterados, sem repetir estados inalterados.

## Benchmarks Criterion

A suíte compara árvores AVL e rubro-negras nas operações de inserção, remoção e
busca. Ela usa árvores com 100, 1.000 e 10.000 chaves; as ordens de inserção e
remoção são crescente, decrescente e alternada. Os benchmarks de busca incluem
acertos e falhas.

Execute a suíte completa:

```sh
cargo bench --bench tree
```

Para uma execução mais rápida, com menos amostras:

```sh
cargo bench --bench tree -- --quick
```

Os relatórios do Criterion são salvos em `target/criterion/`.

Para compilar o executável otimizado do benchmark sem executá-lo:

```sh
cargo bench --bench tree --no-run
```

O Cargo exibe o caminho do executável, geralmente parecido com
`target/release/deps/tree-<hash>`. Execute esse arquivo diretamente e passe as
opções do Criterion depois dele:

```sh
./target/release/deps/tree-<hash> --bench --quick
```

# Árvores AVL e Rubro-Negras

Implementações em Rust de árvores AVL e árvores rubro-negras inclinadas à
esquerda (LLRB), com um visualizador local no navegador usando WebAssembly e
benchmarks com Criterion.

[Read in English](README.md).

## Conteúdo do projeto

- `src/tree/avl`: implementação da árvore AVL.
- `src/tree/rbtree`: implementação da árvore rubro-negra inclinada à esquerda.
- `src/wasm.rs`: interface WebAssembly usada pelo app no navegador.
- `web/`: visualizador estático, sem framework JavaScript.
- `benches/tree.rs`: benchmarks Criterion de inserção, remoção e busca.

O visualizador aceita chaves inteiras, exibe metadados da árvore e permite
acompanhar buscas, rotações e mudanças de cor quadro a quadro. O estado da
árvore fica somente na memória do navegador.

## Requisitos

- Rust estável e Cargo (edição 2024).
- Para o app web: `wasm-pack`, Python 3 e o target Rust
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

O script de desenvolvimento compila o pacote WASM e serve a página em
<http://localhost:8000>:

```sh
./scripts/dev-web.sh
```

O app não tem backend e não faz requisições de rede. Durante o desenvolvimento
é necessário um servidor local porque os navegadores não carregam módulos
WebAssembly a partir de páginas `file://`.

O GitHub Actions publica o site estático no GitHub Pages a cada push para
`main`. Para usar um domínio próprio, configure-o em **Settings → Pages** do
repositório e adicione os registros DNS indicados pelo GitHub Pages. Como o
site é estático, ele também pode ser hospedado em outro serviço de hospedagem
estática.

Mais detalhes: [README web em português](web/README.pt-BR.md) ·
[web README in English](web/README.md).

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
./target/release/deps/tree-<hash> --quick
```

## Benchmark para macOS

Ao enviar uma tag de versão (`v*`), o workflow de release compila executáveis
nativos para Apple Silicon e Intel e os anexa ao GitHub Release correspondente.
Baixe o executável compatível com a arquitetura do Mac e rode-o com opções
opcionais do Criterion, por exemplo `./tree-bench-macos-arm64 --quick`.

Crie e envie uma tag de release assim:

```sh
git tag v0.1.0
git push origin v0.1.0
```

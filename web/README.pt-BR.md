# Tree Study

[Read in English](README.md).

Este é um app web estático. As operações das árvores AVL e rubro-negra são
executadas pelo módulo Rust compilado para WebAssembly; a página usa HTML, CSS e
JavaScript puros para exibir os snapshots e as etapas das operações.

Escolha AVL ou rubro-negra e insira, remova ou busque chaves inteiras. Pressione
Enter no campo de chave para inserir. Use a linha do tempo ou as setas para
revisar os quadros. Se uma operação começar durante a reprodução, ela continua
avançando pelos novos quadros. Os quadros de rotação e recoloração mostram os
snapshots correspondentes, sem repetir estados inalterados.

## Desenvolvimento local

Instale `wasm-pack`, adicione o target Rust `wasm32-unknown-unknown` e, na raiz
do repositório, execute:

```sh
./scripts/dev-web.sh
```

O script compila o pacote WASM e serve a página em <http://localhost:8000>. O
servidor local é necessário apenas para desenvolvimento; o app não faz chamadas
a APIs e não precisa de rede depois que os arquivos estáticos estão disponíveis.

## GitHub Pages

O workflow do Pages compila o pacote WASM e publica o diretório `web/` a cada
push para `main`. Ative o GitHub Pages nas configurações do repositório e
selecione **GitHub Actions** como fonte de build e publicação. Para configurar
um domínio próprio, informe-o nas configurações do Pages e siga as instruções
de DNS exibidas pelo GitHub.

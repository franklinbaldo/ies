---
type: Instructions
title: Instruções para agentes do IES
description: Disciplina de desenvolvimento e validação do projeto Rust
---

# Instruções de desenvolvimento

O [contrato da CLI](contracts/cli.md) é a interface pública primária.
Não reintroduza `agir`, `Action` ou `PageModel` públicos.
Não invente operações pela inspeção de telas: preserve apenas comportamentos
observados e demonstrados nas instalações reais de treinamento do SEI.

Todo Markdown autoral deve ser OKF conforme a [política de CI](docs/ci.md), com Specs e schemas em [`.okf/`](.okf/specs/spec.md).
Nenhum arquivo texto pode ultrapassar 20.000 bytes, inclusive arquivos
de código, testes e lockfiles. Divida por responsabilidade, não minifique.

Antes de enviar qualquer PR:

```sh
uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files
```

Ative os hooks locais com
`uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`.
O [GitHub Actions](.github/workflows/ci.yml) executa esses mesmos gates em
PRs e pushes na `main`, com permissões apenas de leitura.
Não crie um segundo conjunto independente de checks.

## Evidências antes de implementar

Leia o [índice de evidências por operação](contracts/evidencias/README.md),
com as referências `cenario#passo` da matriz do Todos.
Leia o [catálogo OKF de cassettes](docs/cassettes.md) e o conhecimento
[como o SEI funciona](docs/como-o-sei-funciona/README.md).
Um card por revisão aponta para o ZIP original no Internet Archive.
Reproduza offline a evidência antes de implementar; o nome da cassette
não comprova um efeito nem autoriza escrita live.

Não registre segredos, HTML autenticado ou URLs assinadas em commits ou logs.

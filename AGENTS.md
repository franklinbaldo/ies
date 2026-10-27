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

Todo Markdown autoral deve ser OKF conforme a [política de CI](docs/ci.md).
Nenhum arquivo texto pode ultrapassar 20.000 bytes, inclusive arquivos
de código, testes e lockfiles. Divida por responsabilidade, não minifique.

Antes de enviar qualquer PR:

```sh
uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files
```

Ative os hooks locais com `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`.
Não habilite GitHub Actions. O Cloud Build deve executar os mesmos hooks.

Não registre segredos, HTML autenticado ou URLs assinadas em commits ou logs.

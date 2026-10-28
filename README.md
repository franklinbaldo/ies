---
type: Project
title: IES
description: CLI Rust para operar o SEI por contratos de dominio e evidencias reais
---

# IES — CLI Rust para o SEI

Recriação limpa do IES em Rust, orientada pelo contrato público de comandos
do Todos atual e pelas evidências de instalações reais.

**Estado:** primeiro leitor de caixa de processos implementado em Rust.
Outras operações permanecem não implementadas. A leitura da caixa aceita
cassette pública verificada ou uma URL HTTPS assinada de sessão já autenticada;
não implementa login nem paginação.

Não há compatibilidade com o antigo `agir`, `Action` ou `PageModel` público.
O IES Python histórico está preservado no cofre privado
`franklinbaldo/projects-backup`.

O CI público em [GitHub Actions](.github/workflows/ci.yml) executa os mesmos
gates do `prek` usados localmente.

## Experimentos reais do SEI

O [catálogo OKF de cassettes](docs/cassettes.md) registra 495 revisões
distintas observadas em sete instalações de treinamento/homologação.
O [índice por operação](docs/evidencias.md) liga 129 operações da matriz do
Todos a esses cards, registrando agora 0 passos sem card, com revisões ainda a desambiguar.
Essas relações não certificam a implementação IES.
Os ZIPs continuam exclusivamente no Internet Archive. Consulte também
[como o SEI funciona](docs/como-o-sei-funciona/README.md), documentação
empírica independente da arquitetura Python anterior.

## Arquitetura independente do Todos

O [ADR de rotas por evidência](docs/arquitetura/0001-rotas-por-evidencia.md)
separa contratos públicos dos mecanismos REST WSSEI, SOAP autorizado
e navegação Web observada. Antes de ampliar os comandos, o
[experimento de comparação](docs/experimentos/comparar-acessos-sei.md)
deve verificar a equivalência por operação e por instalação.

O [gate de decisões](docs/decisoes/unidade-listar-processos.md) exige
alternativas e evidências para cada operação efetivamente implementada.
Ele preserva a liberdade de escolher a arquitetura interna mais simples
e adequada por instalação, sem portar automaticamente o Todos.

## Primeira leitura

O [guia de replay e leitura da unidade](docs/unidade-primeira-leitura.md)
explica como verificar um ZIP no Internet Archive e executar
`ies unidade listar-processos` com estado de completude explícito.
O programa não expõe links assinados em JSON, nem executa escritas.
O replay offline agora pode consolidar várias páginas já gravadas usando
`--passo N` repetido, sem enviar POST ao SEI e sem inferir páginas ausentes.

## Desenvolvimento

- `cargo test --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`
- `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`

O [contrato de CLI](contracts/cli.md) deve ser completado e aprovado antes
de implementar transporte e operações do SEI. Consulte a [política de CI](docs/ci.md).

---
type: Project
title: IES
description: CLI Rust para operar o SEI por contratos de dominio e evidencias reais
---

# IES — CLI Rust para o SEI

Recriação limpa do IES em Rust, orientada pelo contrato público de comandos
do Todos atual e pelas evidências de instalações reais.

**Estado:** primeiro leitor de caixa de processos implementado em Rust,
com instalação HTTPS autorizada (host e caminho-base) antes de
enviar cookies e leitura HTTP limitada em fluxo.
Outras operações permanecem não implementadas. O replay independente aceita cassette pública verificada;
a consulta experimental de sessão aceita URL HTTPS assinada já autenticada;
não implementa login nem paginação.

Não há compatibilidade com o antigo `agir`, `Action` ou `PageModel` público.
O IES Python histórico está preservado no cofre privado
`franklinbaldo/projects-backup`.

O CI público em [GitHub Actions](.github/workflows/ci.yml) executa os mesmos
gates do `prek` usados localmente.

## Documentação do SEI

O [catálogo OKF de cassettes](docs/cassettes.md) registra 495 revisões
capturadas em sete instalações de treinamento/homologação.
O [índice por operação](docs/evidencias.md) liga 129 operações da matriz do
Todos a esses cards, registrando agora 0 passos sem card, com revisões ainda a desambiguar.
Essas relações não certificam a implementação IES.
`Cassette`, `OperationEvidence` e `SEIObservation` são categorias
de **documentos OKF**, não tipos ou componentes do runtime.
Os ZIPs continuam exclusivamente no Internet Archive. Consulte também
[como o SEI funciona](docs/como-o-sei-funciona/README.md), documentação
empírica independente da arquitetura Python anterior.

## Arquitetura independente do Todos

O [ADR de rotas por evidência](docs/arquitetura/0001-rotas-por-evidencia.md)
separa contratos públicos dos mecanismos REST WSSEI, SOAP autorizado
e cliente Web do SEI. Antes de ampliar os comandos, o
[experimento de comparação](docs/experimentos/comparar-acessos-sei.md)
deve verificar a equivalência por operação e por instalação.

O [gate de decisões](docs/decisoes/unidade-listar-processos.md) exige
alternativas e evidências para cada operação efetivamente implementada.
Ele preserva a liberdade de escolher a arquitetura interna mais simples
e adequada por instalação, sem portar automaticamente o Todos.

## Identidades do SEI

A [instalação SEI](docs/interfaces/instalacoes-e-versoes.md) é o
endereço-base HTTPS autorizado, com `VersaoSei` declarada opcional.
Órgão é a instituição administrada no SEI/SIP; unidade é seu
setor, ativo na sessão. Uma instalação pode conter vários órgãos.
A compatibilidade é inferida dos **testes por operação e instalação**,
nunca apenas do número de versão. Não existe um framework de
compatibilidade nem uma confiança percentual inventada.

## Primeira leitura

O [guia de replay e leitura da unidade](docs/unidade-primeira-leitura.md)
explica como verificar um ZIP no Internet Archive e executar
`ies unidade listar-processos` com estado de completude explícito.
O domínio usa tipos Rust (`Processo`, `IdProcedimento`,
`Tabela<T>` e `Caixa`). A cobertura é validada na paginação;
`completa: bool` é **derivação da borda**, não estado duplicado.
O [golden JSON v1](tests/fixtures/caixa-v1.json) protege a saída.
O programa não expõe links assinados em JSON, nem executa escritas.
O replay offline agora pode consolidar várias páginas já gravadas usando
`--passo N` repetido, sem enviar POST ao SEI e sem inferir páginas ausentes.

## Desenvolvimento

Os checkers de infraestrutura são um `xtask` Rust sem dependências
externas; o `prek` os executa sem compilar a CLI. O GitHub Actions
usa cache de Rust pinado por SHA e testa todos os alvos do workspace.

- `cargo test --locked --workspace --all-targets`
- `cargo fmt --all -- --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings -W clippy::pedantic`
- `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`
- `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`

O [contrato de CLI](contracts/cli.md) deve ser completado e aprovado antes
de implementar transporte e operações do SEI. Consulte a [política de CI](docs/ci.md).

## Entradas de execução

- `ies processo NUP` e `ies unidade listar-processos` são contratos de
  domínio e não recebem arquivos de cassette nem URLs assinadas.
- `ies replay CENARIO` reproduz uma captura verificada do catálogo OKF
  sem abrir conexão com o SEI; [guia de replay](docs/replay.md).
- `ies sessao ler-caixa` é uma ferramenta experimental para uma sessão
  Web previamente autenticada, com origem e credenciais privadas.

---
type: ImplementationDecision
title: Listagem local de unidades
operation: unidade.listar
implementation: partial
status: adopted-for-scope
scope: ies unidade listar sobre snapshots SQLite por instalação e unidade da sessão local
selected_strategy: snapshot-validado-local
alternatives: snapshot-validado-local,consulta-web-por-pesquisa,unidade-inferida-sem-escopo
evidence_level: testes-cli-sinteticos-e-indice-local
live_validation: not-run
---

# Listagem local de unidades (#244)

## Alternativas consideradas

- **snapshot-validado-local:** reutilizar o índice SQLite com chave de
  instalação e unidade já observada na URL privada do Controle.
- **consulta-web-por-pesquisa:** pedir ao SEI a cada uso e descartar o cache.
- **unidade-inferida-sem-escopo:** consultar registros de qualquer unidade.

## Comparação

A escolha local usa **zero** requisições, preserva os registros
integralmente guardados e não mistura unidades. Uma consulta Web
por pesquisa desperdiçaria requisições, e ignorar a unidade ativa
misturaria destinos de sessões distintas.

## Decisão

Publicar a lista já guardada, distinguindo snapshot ausente de lista vazia.
A origem e a unidade são verificadas apenas nos arquivos da sessão,
sem publicar URL assinada ou cookies e sem alegar sessão remota válida.
`--unidade` sem confirmação da tela é recusada, não ignorada.
A captura Web para índice ausente e a atualização integral seguem
pendentes da issue #244.

## Evidências verificáveis

- [Contrato](../../contracts/cli/unidade-listar.md)
- [Evidência](../../contracts/evidencias/unidade-listar.md)
- [Teste Rust](../../tests/cli_unidade_indice.rs)
- [Implementação](../../src/unidade_pesquisar_cli.rs)
- [Observação da lupa](../como-o-sei-funciona/lupa-de-unidades.md)

## Limitações e revisão

CI e gates devem ser medidos no head desta PR. RED local e live
UnB/GO seguem NOT_RUN se não forem efetivamente executados.
A implementação permanece parcial até a leitura completa da lupa
e o replay das cassettes serem validados.

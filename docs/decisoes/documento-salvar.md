---
type: ImplementationDecision
title: Prévia offline sem liberar escrita remota de documento
operation: documento.salvar
implementation: partial
status: adopted-for-scope
scope: previa offline do par DOC.vN.html (foto) e DOC.v(N+1).html (edicao), sem indice, configuracao nem POST no SEI e sem verificar a base (#207)
selected_strategy: local-preview
alternatives: local-preview,web-editor-write,rest-document-update
evidence_level: testes-sinteticos-de-dominio-e-cli-e-previa-local-sobre-foto-lida-ao-vivo
live_validation: previa-local-pass-unb-4-0-9-escrita-remota-nao-executada
---

# Prévia local antes de escrita Web

## Alternativas consideradas

- **local-preview:** conferir em disco o par `DOC.vN.html` (a foto que o
  SEI serviu) e `DOC.v(N+1).html` (a edição), sem rede, índice nem
  configuração.
- **web-editor-write:** abrir o editor pode remover assinaturas; falta
  identificar seções e retorno de versão por instalação.
- **rest-document-update:** módulos REST não são universais e não há
  evidência de semântica de escrita equivalente.

## Comparação

A prévia local reduz perda de dados, mas não satisfaz a escrita. O Web editor
poderia ser o destino de escrita somente depois de ensaios de efeitos e
concorrência. O REST depende do módulo e não é alternativa de fallback.

## Decisão

Somente a prévia é implementada agora. Desde #207 (05/11/2026, decisão
do proprietário que **substitui** a cópia de trabalho de #88 com
`.DOC.vN.base.html` e `.DOC.vN.meta.json`), a base é a própria foto
`DOC.vN.html` gravada por `ies documento DOC --saida`
([foto idempotente](documento-obter/saida-idempotente.md)); a edição é
`DOC.v(N+1).html`. Base ausente, edição ausente ou igual à base são
recusas tipadas. A prévia é **puramente offline** e **não verifica a
base**: diz isso no JSON (`base_verificada: false`) e no texto ("base
não verificada"). `--confirmar` falha fechado, sem abrir editor, POST,
retry ou fallback.

Revisão de 05/11/2026 (decisão do proprietário, antes do merge de
#219): a primeira versão conferia a base pelo SHA-256 numa tabela
`foto` do índice local (`base_versao_nao_registrada`,
`base_versao_divergente`). Saiu por ser redundante. Reler o documento
com `--saida` já compara a foto local com o HTML servido e recusa com
`foto_local_divergente`; o salvar remoto (#89) lerá a versão N do SEI
para confirmar que ainda é a atual e comparará o hash com a vN local.
A prévia, que não fala com o SEI, não tem como afirmar nada sobre a
base e por isso não finge: um campo explícito foi preferido a uma nota
só no texto porque agentes leem o JSON.

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-salvar.md)
- [Matriz histórica](../../contracts/evidencias/documento-salvar.md)
- [Testes de integração CLI](../../tests/cli_contract.rs) e
  [do par local](../../tests/cli_documento_saida.rs)
- [Testes de domínio Rust](../../src/copia_documento/tests.rs)
- [Ensaio live da prévia sobre a foto lida](../validacao-live/2026-11-05-documento-saida-idempotente.md)

## Limitações e revisão

A prévia foi exercida sobre uma foto lida ao vivo na UnB 4.0.9; a
escrita Web não foi testada em instalação alguma e o fluxo completo da
issue #89 não está concluído. Uma foto alterada à mão passa pela
prévia; quem a recusa é a releitura ou o salvar remoto. Rever quando
#89 observar o editor e a promoção de versão.

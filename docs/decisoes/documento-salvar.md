---
type: ImplementationDecision
title: Prévia offline sem liberar escrita remota de documento
operation: documento.salvar
implementation: partial
status: adopted-for-scope
scope: previa local do par DOC.vN.html (foto registrada no indice) e DOC.v(N+1).html (edicao), sem POST no SEI (#207)
selected_strategy: local-preview
alternatives: local-preview,web-editor-write,rest-document-update
evidence_level: testes-sinteticos-de-dominio-e-cli-e-previa-local-sobre-foto-lida-ao-vivo
live_validation: previa-local-pass-unb-4-0-9-escrita-remota-nao-executada
---

# Prévia local antes de escrita Web

## Alternativas consideradas

- **local-preview:** conferir em disco o par `DOC.vN.html` (a foto que o
  SEI serviu) e `DOC.v(N+1).html` (a edição), com o SHA-256 da foto
  registrado no índice local, sem fazer rede.
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
([foto idempotente](documento-obter/saida-idempotente.md)), conferida
pelo SHA-256 que o índice local registrou; a edição é `DOC.v(N+1).html`.
Base não registrada, alterada, edição ausente ou igual são recusas
tipadas. O índice foi escolhido para o SHA-256 em vez de um manifesto
oculto porque já guarda o observado por instalação e não espalha
arquivos. `--confirmar` falha fechado, sem abrir editor, POST, retry ou
fallback.

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
issue #89 não está concluído. Sem configuração de sessão ou com o
índice desligado, nenhuma base é aceita. Rever quando #89 observar o
editor e a promoção de versão.

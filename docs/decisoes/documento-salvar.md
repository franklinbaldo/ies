---
type: ImplementationDecision
title: Prévia offline sem liberar escrita remota de documento
operation: documento.salvar
implementation: partial
status: adopted-for-scope
scope: copia local versionada e previa segura sem POST no SEI
selected_strategy: local-preview
alternatives: local-preview,web-editor-write,rest-document-update
evidence_level: testes-sinteticos-de-dominio-e-cli
live_validation: nao-executada
---

# Prévia local antes de escrita Web

## Alternativas consideradas

- **local-preview:** conferir em disco número opaco, versão, identidade,
  hash da base e edição local sem fazer rede, preservando as gerações.
- **web-editor-write:** abrir o editor pode remover assinaturas; falta
  identificar seções e retorno de versão por instalação.
- **rest-document-update:** módulos REST não são universais e não há
  evidência de semântica de escrita equivalente.

## Comparação

A prévia local reduz perda de dados, mas não satisfaz a escrita. O Web editor
poderia ser o destino de escrita somente depois de ensaios de efeitos e
concorrência. O REST depende do módulo e não é alternativa de fallback.

## Decisão

Somente a prévia é implementada agora. Cópias de editor só podem ser
criadas pelo adaptador que tenha verificado fonte, assinatura e seções;
o helper Rust recusa visualizações históricas. `--confirmar` falha
fechado, sem abrir editor, POST, retry ou fallback.

## Evidências verificáveis

- [Contrato](../../contracts/cli/documento-salvar.md)
- [Matriz histórica](../../contracts/evidencias/documento-salvar.md)
- [Testes de integração CLI](../../tests/cli_contract.rs)
- [Testes de domínio Rust](../../src/copia_documento.rs)

## Limitações e revisão

Validação offline sintética apenas. Não houve teste Web em UnB, UFRJ,
Goiás ou PEN; o fluxo completo da issue #89 não está concluído.

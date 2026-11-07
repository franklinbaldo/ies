---
type: OperationEvidence
title: documento.incluir-externo
operation: documento.incluir-externo
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 0
unresolved_steps: 0
---

# documento.incluir-externo — cenário derivado #332

**Esta é uma operação IES nova e derivada.** A matriz do Todos no commit indicado não indexa roteiro específico com este nome, por isso **zero passos atribuídos**: não inventar uma cassette nem declarar paridade. A matriz é contexto do projeto, não prova da implementação ou da escrita.

O oráculo comportamental direto é [Todos, documento_externo.py](https://github.com/franklinbaldo/todos/blob/main/src/todos/documento_externo.py), com observação em **UnB de treinamento, SEI 4, 11/10/2026**: seleção do tipo, cadastro externo, **upload temporário** e **envio de cadastro**, seguidos de confirmação da árvore. No IES Rust esta capacidade remota **não está implementada**. A PR #352 cobre exclusivamente inspeção local (nome, extensão, bytes, assinatura PDF e SHA-256) e recusa `--confirmar` antes da sessão/rede.

O caso [Todos #1246](https://github.com/franklinbaldo/todos/issues/1246) mostra que a árvore pode não comprovar os metadados do anexo embora a tela de alteração publique o arquivo. Isto é um cenário a traduzir em teste Rust, **não PASS do IES**.

- [Teste de CLI local](../../tests/cli_documento_incluir_externo.rs)
- [Contrato parcial](../cli/documento-incluir-externo.md)
- [Relatório offline e live NOT_RUN](../../docs/validacao-live/2026-11-06-documento-incluir-externo-local.md)

Sem novas requisições ou efeitos remotos nesta fatia. Não confundir a classificação `writes:true` (natureza futura da capacidade) com execução ou confirmação de escrita.

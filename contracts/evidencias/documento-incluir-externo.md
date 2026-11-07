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

## Evidência adicional do acervo (somente leitura)

- [Teste do Todos na UnB, SEI 4](https://github.com/franklinbaldo/todos/blob/main/tests/test_documento_juntar_010.py): a tela `documento_receber` é recarregada após a escolha em `selSerie`, com `hdnIdSerie` e `hdnFlagDocumentoCadastro=1`. O POST de recarga não equivale ao POST de cadastro definitivo. O teste registra o recibo sintético `94e336aa#prova.pdf#application/pdf#68#11/10/2026 11:07:50#`, derivado do formato observado.
- [Cassette PEN3 de prévia](../../cassettes/pen3/documento-juntar-previa-pen3--ec754357a3c7.md), instalação SEI 5.0.4: **9** interações da captura (5 GET, 4 POST), não 9 requisições do IES. O [teste do Todos para SEI 5](https://github.com/franklinbaldo/todos/blob/main/tests/test_documento_juntar_sei5_010.py) registra que a tela desta versão publica `new infraUpload('frmAnexos', URL, true)`, com terceiro argumento booleano, diferentemente da forma de dois argumentos no SEI 4.
- [Oráculo `documento_externo.py`](https://github.com/franklinbaldo/todos/blob/main/src/todos/documento_externo.py): a resposta do upload temporário tem seis campos delimitados por `#` (o último vazio) ou `ERRO#motivo`; o envio definitivo usa `hdnAnexos` com colunas determinadas pelo script da tela, não por uma linha fixa inventada. O charset do nome do arquivo segue a página servida.
- **Classificação dos efeitos:** o POST de recarga é de leitura pela semântica observada, mas o upload temporário e o POST de cadastro são duas mutações distintas; o recibo do primeiro não confirma o segundo. Nenhuma dessas cassettes documenta um upload novo feito pelo IES.

Esta leitura da evidência pública **não substitui** observação atual do formulário em TREINAMENTO, nem valida status/limites, URLs assinadas, recusa de redirects, exatidão de contagens reais ou a releitura de um documento novo. `--confirmar` segue fechado.

O caso [Todos #1246](https://github.com/franklinbaldo/todos/issues/1246) mostra que a árvore pode não comprovar os metadados do anexo embora a tela de alteração publique o arquivo. Isto é um cenário a traduzir em teste Rust, **não PASS do IES**.

- [Teste de CLI local](../../tests/cli_documento_incluir_externo.rs)
- [Contrato parcial](../cli/documento-incluir-externo.md)
- [Relatório offline e live NOT_RUN](../../docs/validacao-live/2026-11-06-documento-incluir-externo-local.md)

Sem novas requisições ou efeitos remotos nesta fatia. Não confundir a classificação `writes:true` (natureza futura da capacidade) com execução ou confirmação de escrita.

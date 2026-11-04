---
type: SEIObservation
title: Replay do acervo público pelos leitores de documento DOC
description: Tela da pesquisa rápida, link do conteúdo por ícone e conteúdo documento_visualizar das cassettes públicas das sete instalações, lidos offline pelos leitores de ies documento DOC, 04/11/2026
---

# Replay do acervo para `documento DOC` — 04/11/2026

Teste ignorado `replay_de_documento_no_acervo`
(`src/processo_obter/acervo_documento.rs`), mesmos leitores da
[leitura Web](../../contracts/cli/documento-obter-web.md), sem rede:

    IES_ACERVO_SAIDA=saida.md cargo test --bin ies replay_de_documento_no_acervo -- --ignored

Cada card é conferido (SHA-1 do ZIP, SHA-256 do membro) como no
[replay de processo NUP](2026-11-04-processo-obter-acervo.md). Só formas
e contagens; nenhuma URL.

## Resultado

280 cassettes lidas.

| Leitor | Classe | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| conteúdo | título e texto lidos | 38 | 0 | 0 | 12 | 12 | 14 | 15 |
| link do conteúdo | `documento_cancelado` → `vazio` | 0 | 3 | 3 | 3 | 3 | 3 | 3 |
| link do conteúdo | `documento_email` → `documento_visualizar` | 0 | 6 | 6 | 6 | 6 | 0 | 6 |
| link do conteúdo | `documento_html` → `documento_download_anexo` | 0 | 19 | 0 | 28 | 23 | 18 | 17 |
| link do conteúdo | `documento_interno` → `documento_visualizar` | 1436 | 971 | 121 | 84 | 93 | 93 | 102 |
| link do conteúdo | `documento_interno` → `vazio` | 68 | 15 | 2 | 0 | 0 | 0 | 0 |
| link do conteúdo | `documento_pdf` → `documento_download_anexo` | 43 | 16 | 0 | 0 | 0 | 9 | 7 |
| link do conteúdo | `documento_txt` → `documento_download_anexo` | 0 | 0 | 18 | 3 | 0 | 3 | 3 |
| link do conteúdo | `documento_zip` → `vazio` | 0 | 0 | 0 | 0 | 0 | 0 | 6 |
| tela da pesquisa rápida | `endereco_da_arvore_de_outro_documento` | 84 | 57 | 9 | 5 | 5 | 7 | 5 |
| tela da pesquisa rápida | documento selecionado na árvore | 2 | 1 | 1 | 1 | 1 | 1 | 1 |

## Leitura

- **Tela da pesquisa rápida:** as 8 telas abertas por número SEI
  (`documento-numero-e-busca-*` nas sete instalações e
  `documento-propriedades-unb`) selecionam o documento na árvore. As
  demais (172) vieram de pesquisa por NUP e são recusadas como
  `endereco_da_arvore_de_outro_documento`: um número de processo nunca
  passa por documento.
- **Link do conteúdo:** nenhuma recusa. Interno e e-mail publicam
  `documento_visualizar`; PDF, TXT e HTML recebido,
  `documento_download_anexo`; cancelado e ZIP (PEN4), `''`. Interno com
  `''` (UnB 68, UFRJ 15, GO 2): sem acesso à sessão gravada.
- **Conteúdo:** as 91 respostas `documento_visualizar` gravadas (UnB e
  PEN1 a PEN4; UFRJ e GO não gravaram nenhuma) passam pelo título e pelo
  texto. Um levantamento à parte pelo `--html-privado` leu também as 3
  respostas `documento_download_anexo` gravadas (HTML recebido, PEN1, PEN2
  e PEN4), cujo título traz
  o número do órgão de origem, não o da árvore: por isso externos não
  entram nesta fatia.
- Duplicidade de número na árvore: nenhuma ocorrência no acervo; a recusa
  é coberta por fixture sintética.

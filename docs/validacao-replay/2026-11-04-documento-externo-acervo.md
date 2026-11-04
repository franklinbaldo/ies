---
type: SEIObservation
title: Replay do acervo público pelo leitor do anexo de documento externo
description: Links documento_download_anexo das árvores e respostas gravadas do anexo nas cassettes públicas das sete instalações, lidos offline pelos leitores de ies documento DOC externo, 04/11/2026
---

# Replay do acervo para o documento externo — 04/11/2026

Mesmo teste ignorado do [replay de `documento DOC`](2026-11-04-documento-obter-acervo.md)
(`replay_de_documento_no_acervo`, `src/processo_obter/acervo_documento.rs`),
agora também com a operação `documento_download_anexo`; leitores do
[documento externo](../../contracts/cli/documento-obter-externo.md), sem
rede, cada card conferido (SHA-1 do ZIP, SHA-256 do membro):

    IES_ACERVO_SAIDA=saida.md cargo test --bin ies replay_de_documento_no_acervo -- --ignored

## Resultado

280 cassettes lidas; só as linhas do externo (as demais são as do
replay da fatia 1, sem mudança).

| Leitor | Classe | unb | ufrj | go | pen1 | pen2 | pen3 | pen4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| anexo | `text/html` lido, com texto | 0 | 0 | 0 | 1 | 1 | 0 | 1 |
| link do conteúdo | `documento_html` → `documento_download_anexo` | 0 | 19 | 0 | 28 | 23 | 18 | 17 |
| link do conteúdo | `documento_pdf` → `documento_download_anexo` | 43 | 16 | 0 | 0 | 0 | 9 | 7 |
| link do conteúdo | `documento_txt` → `documento_download_anexo` | 0 | 0 | 18 | 3 | 0 | 3 | 3 |
| link do conteúdo | `documento_zip` → `vazio` | 0 | 0 | 0 | 0 | 0 | 0 | 6 |

## Leitura

- **Link:** os 252 `.src` `documento_download_anexo` passam pela
  conferência nova (um `id_anexo` decimal, `infra_hash` de 64 hex, chaves
  canônicas), nenhum recusado. Forma: `acao`, `acao_origem`, `id_anexo`,
  `arvore`, `infra_sistema`, `infra_unidade_atual`, `infra_hash`; sem
  `id_documento`. ZIP (PEN4) segue sem link.
- **Anexo:** as 3 respostas gravadas (`documento-numero-e-busca-*` de
  PEN1, PEN2 e PEN4) são HTML recebido de outro órgão pela tramitação
  (título `SEI/<órgão de origem> - <número> - <tipo>`), status 200,
  `Content-Type: text/html;;charset=iso-8859-1`; nenhuma tem a tela de
  login nem o contêiner de exceção, e todas rendem o texto visível.
- **Fora do alcance do acervo:** as cassettes guardam só `content-type` e
  `location`; `Content-Disposition`, `Content-Length` e respostas PDF ou
  TXT não foram gravados. O ensaio live da PEN1 os observa.

---
type: ImplementationDecision
title: Ler documento externo pelo GET binário do link documento_download_anexo da árvore
operation: documento.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de documento.obter (Refs #60 fatia 3); o .src documento_download_anexo do nó deixa de ser recusa e vira um GET binário do link assinado da árvore, com tipos observados (PDF, texto, HTML recebido), limite configurável, SHA-256 em fluxo, parcial 0600 em --saida e texto só dos tipos de texto; ZIP, sem acesso e cancelado continuam sem link na árvore
selected_strategy: web-get-binario-do-src-da-arvore
alternatives: web-get-binario-do-src-da-arvore,web-get-html-com-limite-de-8-mb,web-zip-do-processo,web-url-montada-por-id-anexo
evidence_level: acervo-publico-oraculo-todos-testes-sinteticos-e-live-pen1
live_validation: ver-relatorio-live-pen1-5-0-4
---

# Ler documento externo

Complementa a [decisão de `documento.obter`](../documento-obter.md);
contrato em [documento externo](../../../contracts/cli/documento-obter-externo.md).

## Alternativas consideradas

- **web-get-binario-do-src-da-arvore** (escolhida): o `Nos[i].src`
  `documento_download_anexo` que a árvore publica para o nó já conferido,
  por um caminho binário próprio do cliente HTTPS único.
- **web-get-html-com-limite-de-8-mb:** reaproveitar o GET de HTML. Só
  serve ao HTML recebido; PDF seria `tipo_de_conteudo_nao_html` e o
  corpo inteiro ficaria na memória.
- **web-zip-do-processo:** "Gerar arquivo ZIP do processo" e extrair o
  documento. Mais requisições, efeito de geração no servidor e o processo
  inteiro para ler um documento.
- **web-url-montada-por-id-anexo:** montar `documento_download_anexo`
  pelo `id_anexo`. Vedado: link sem assinatura derruba a sessão (ADR 0003).

## Comparação

- **Forma do link.** As 252 ocorrências de `documento_download_anexo` no
  acervo (PDF, TXT e HTML recebido, seis instalações) passam pela mesma
  conferência do link interno, com um único `id_anexo` decimal no lugar
  do `id_documento` ([replay](../../validacao-replay/2026-11-04-documento-externo-acervo.md)).
  O link não traz o id do documento: a ligação é o nó.
- **Respostas gravadas.** Só três (PEN1, PEN2, PEN4), todas HTML recebido
  com `Content-Type: text/html;;charset=iso-8859-1` (parâmetro vazio) e
  sem `Content-Disposition`/`Content-Length` gravados (as cassettes
  guardam só `content-type` e `location`). O oráculo Todos mediu na UnB
  `application/pdf` com `Content-Disposition: inline; filename="…"` e
  registra que o cabeçalho não é garantido em toda instalação.
- **Memória e disco.** O binário é lido em blocos de 64 KiB com SHA-256;
  com `--saida` vai para um parcial criado novo em 0600 e só ganha o
  nome final (por `hard_link`, que nunca sobrescreve) depois de todos os
  documentos lidos; sem `--saida`, PDF só rende metadados.
- **Nome.** O nome do servidor é publicado saneado e nunca vira caminho;
  a extensão vem do `Content-Type` permitido.

## Decisão

GET binário do link da árvore, lista fechada de tipos pela evidência
(`application/pdf`, `text/plain`, `text/html`), cabeçalhos conferidos
como únicos, limite `IES_LIMITE_ANEXO_BYTES` (padrão 64 MiB; texto até
8 MB) antes e durante a leitura, HTML de login ou exceção do SEI
recusado. Um cabeçalho com bytes fora de UTF-8 passa a ser lido como
ISO-8859-1 no transporte, em vez de falha de rede.

## Evidências verificáveis

- Testes: `src/anexo_documento/tests.rs`,
  `src/transporte/tests_resposta_binaria.rs`,
  `src/processo_obter/tests_documento_externo.rs` e
  `link_do_anexo_fora_da_forma_e_recusado` em
  `src/arvore_processo/tests_conteudo.rs`.
- Replay: `replay_de_documento_no_acervo`
  (`src/processo_obter/acervo_documento.rs`).
- Live: [PEN1 5.0.4](../../validacao-live/2026-11-04-documento-externo.md).

## Limitações e revisão

- Outros tipos (DOCX, imagens, planilhas) são recusados até serem
  observados; ampliar a lista é uma revisão desta decisão.
- `text/plain` sem charset e fora de UTF-8 não rende texto (com
  `--saida`, o arquivo é gravado como veio).
- Rever se o GET do anexo marcar o documento como visitado, ou se uma
  instalação responder o anexo com redirect.

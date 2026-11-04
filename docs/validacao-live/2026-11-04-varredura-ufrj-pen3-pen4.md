---
type: SEIObservation
title: Varredura live de leitura na UFRJ 4.1.5, PEN3 e PEN4 5.0.4
description: PASS/FAIL/NOT_RUN de processo NUP (frio e quente), processo buscar, documento DOC (interno, externo PDF e HTML, --processo, --versao N, --saida) e documento buscar na UFRJ 4.1.5, PEN3 e PEN4 5.0.4, com a correção do processo visitado na lista da pesquisa de processos, 04/11/2026
---

# Varredura live de leitura — UFRJ, PEN3 e PEN4 — 04/11/2026

Rodada só de leitura para o #176 e os critérios live ainda abertos do
#58, #60, #88 e #61, com o binário release do ramo `slice-varredura-a`
(base `7a8b332`, antes da correção descrita abaixo). Complementa os
ensaios de [processo NUP](2026-11-03-processo-obter-arvore.md),
[processo buscar](2026-11-04-processo-buscar.md),
[documento DOC](2026-11-04-documento-obter.md),
[externo](2026-11-04-documento-externo.md),
[versões](2026-11-04-documento-versoes.md) e
[documento buscar](2026-11-04-documento-buscar.md).

## Condições

- **Instalações:** UFRJ 4.1.5 (`teste.sei.tic.ufrj.br`, unidade `PR-P`),
  PEN3 e PEN4 5.0.4 (`sei.orgao3`/`sei.orgao4.tramita.processoeletronico.gov.br`,
  unidade `TESTE`). Sessões e índices existentes de `ies login` (contas
  públicas de [treinamento](../interfaces/credenciais-de-treinamento.md)),
  arquivos `0600` numa pasta `0700` fora do repositório; nenhuma precisou
  relogar.
- **Efeito avaliado:** só GET e os POST da pesquisa rápida e da Pesquisa.
  Abrir o processo pela árvore pode registrar recebimento ou visita (como
  no #59); a pesquisa lembra página e filtro na conta; o 302 da Pesquisa
  não é seguido. Nenhum `--confirmar`, nenhuma escrita.
- **Orçamento declarado:** ≤ 20 comandos por instalação, ≥ 30 s entre o
  fim de um comando e o início do seguinte na UFRJ (outro agente usava a
  mesma instalação) e ≥ 22 s na PEN3 e na PEN4, sem retry; parar a
  instalação no `bloqueio_de_seguranca_da_instalacao`. Usados: UFRJ 15
  comandos e 48 requisições; PEN3 8 e 24 (parou no bloqueio); PEN4 15 e
  47. Requisições contadas por um relay CONNECT local (um túnel por
  requisição); o campo `requisicoes` da saída coincidiu com os túneis em
  todos os comandos.
- **Termos:** NUPs e números SEI tirados do índice local (colhido do
  Controle), das árvores lidas na rodada, das listas da própria pesquisa
  e das cassettes públicas (ícones `documento_pdf` nas árvores). Só
  formas, números e contagens abaixo; nenhuma URL assinada, `infra_hash`
  ou cookie foi impresso ou versionado (os textos gravados citam só a URL
  pública de conferência do SEI, sem hash).

## `processo NUP` (#176, #59)

| Instalação | NUP | Frio | Quente | Veredito |
| --- | --- | --- | --- | --- |
| UFRJ | `99079.901789/2026-19` | 5 req., `indice_trabalhar`, `unidade` `PR-P` | 4 req., `indice_link`, `unidade: null` | **PASS** |
| PEN3 | `98390.000014/2026-29` | 5 req., `indice_trabalhar` | 4 req., `indice_link` | **PASS** |
| PEN4 | `98490.000070/2026-35` | 5 req., `indice_trabalhar` | 4 req., `indice_link` | **PASS** |

As três publicaram árvore, `aberto_em` (uma unidade, sem atribuição),
andamentos (1, 13 e 1 de 1, 13 e 1 declarados), propriedades pela
`procedimento_alterar` (nível de acesso `publico`; na PEN3 `restrito`
com hipótese legal), marcadores (0 de 0) e a natureza de cada documento
(`icone` `documento_interno` na UFRJ e na PEN4, `documento_pdf` nos dois
da PEN3, `cancelado: false`). Frio = link de trabalho colhido do Controle
\+ árvore + 3 telas; quente = link da árvore + 3 telas, como no contrato
(`unidade` não é lida nesse caminho).

## `processo buscar` (#58)

| Instalação | Termo | Esperado | Saída do IES | Req. | Veredito |
| --- | --- | --- | --- | ---: | --- |
| UFRJ | `99079.901786/2026-85` (na caixa) | `processo_unico` | `processo_unico`, id 5126 (o mesmo do `documento 0003105`) | 3 | **PASS** |
| UFRJ | `99079.901784/2026-96` (fora da caixa e do índice) | 302 ou lista | `processo_unico`, id 5114 | 3 | **PASS** |
| UFRJ | `99079.9017` | `lista` | 10 linhas, total 62, faixa 1–10 | 3 | **PASS** |
| UFRJ | `99079.9017 --pagina 2` | faixa 11–20 | 10 linhas, total 62, faixa 11–20 | 4 | **PASS** |
| UFRJ | `99079.901789/2026-19` (na caixa, gerado no dia) | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** (ver observação) |
| UFRJ | `99079.900039/2025-49` (outra unidade) | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** (resposta observada) |
| PEN4 | `98490.000070/2026-35` (na caixa) | `processo_unico` | `processo_unico`, id 1349 (o da árvore) | 3 | **PASS** |
| PEN4 | `98490.000055/2026-97` (fora da caixa) | 302 ou lista | `processo_unico`, id 1294 (o do `documento 0000820`) | 3 | **PASS** |
| PEN4 | `98490.0000` | `lista` | `html_de_pesquisa_nao_interpretado`, saída 1 | 3 | **FAIL → corrigido offline** |
| PEN4 | `98490.0000 --pagina 2` | faixa 11–20 | o mesmo erro, na página 1 (nenhum POST de página) | 3 | **FAIL** (mesma causa) |
| PEN4 | `99999.999999/2026-00` | resposta observada | `sem_resultado`, saída 0 | 3 | **PASS** |
| PEN3 | controle (NUP na caixa) | `processo_unico` | — | — | **NOT_RUN**: a PEN3 parou no bloqueio antes |

## `documento DOC` (#60, #88)

| Instalação | Pedido | Caminho | Req. | Resultado | Veredito |
| --- | --- | --- | ---: | --- | --- |
| UFRJ | `0003105` (interno, frio) | pesquisa rápida | 4 | texto, versão 1 de 1 | **PASS** |
| UFRJ | `0003109 --saida` (interno, quente) | link da árvore | 2 | `0003109.txt` 0600, `versao_atual` 2 do rodapé, sem cópia (`edicao_nao_publicada_na_arvore`) | **PASS** |
| UFRJ | `0003109 --versao 1` | link da árvore + barra + lista | 3 | texto da versão 1, `versao_atual: 2` pela lista | **PASS** |
| UFRJ | `0003105 --processo 99079.901786/2026-85 --saida` | link da árvore | 2 | `0003105.txt` e cópia `0003105.v1.html` + base + manifesto 0600, sem `infra_hash` | **PASS** |
| UFRJ | `0000094 --saida` (PDF de outra unidade, cassette) | pesquisa rápida | 3 | `conteudo_nao_publicado_na_arvore` | **PASS** (recusa tipada) |
| PEN3 | `0001003 --saida` (PDF, quente) | link da árvore + anexo | 2 | `0001003.pdf` 0600, `application/pdf` `inline`, 36 998 bytes, SHA-256 conferido, começa por `%PDF-1.4` | **PASS** |
| PEN3 | `0001018 --processo 98390.000036/2026-99 --saida` | link colhido + árvore + conteúdo | 3 | `0001018.txt`, `versao_atual` 3 do rodapé, `documento_assinado` | **PASS** |
| PEN3 | `0001018 --versao 2` | link da árvore + barra + lista | 3 | texto da versão 2, `versao_atual: 3` | **PASS** |
| PEN3 | `0001021 --saida` (PDF da cassette `processo-escrita-ciclo`) | pesquisa rápida | 1 | `pesquisa_rapida_nao_abriu_o_documento` (200 sem 302) | resposta observada; o documento provavelmente foi excluído pelo ciclo do Todos (não verificado) |
| PEN3 | `0001054` (interno, frio) | pesquisa rápida | 3 | 403 do Cloudflare na terceira requisição: `bloqueio_de_seguranca_da_instalacao`; PEN3 encerrada | **NOT_RUN** |
| PEN4 | `0000820 --saida` (PDF, frio, fora da caixa) | pesquisa rápida | 4 | `0000820.pdf` 0600, `application/pdf` `inline`, 8 952 bytes, `%PDF-1.3` | **PASS** |
| PEN4 | `0000872 --saida` (externo, frio) | pesquisa rápida | 4 | `0000872.html`, `text/html` `inline`, 80 bytes (`documento_html`) | **PASS** |
| PEN4 | `0000851 --processo 98490.000070/2026-35 --saida` | link da árvore | 2 | `0000851.txt`, `versao_atual` 3 | **PASS** |
| PEN4 | `0000851 --versao 1 --saida` (pasta com `0000851.txt`) | link + barra + versão | 3 | `arquivo_texto_documento_ja_existe` depois das 3 requisições | achado 2 |
| PEN4 | `0000851 --versao 1` | link + barra + versão | 3 | texto da versão 1, `versao_atual: 3` | **PASS** |
| PEN4 | `0000854 --saida` (PDF da cassette `processo-escrita-ciclo`) | pesquisa rápida | 1 | `pesquisa_rapida_nao_abriu_o_documento` | resposta observada, como `0001021` |

Orçamentos iguais aos do contrato: pesquisa rápida com o `action`
guardado G = 3 (+1 do conteúdo ou do anexo), link da árvore 1 + 1,
`--versao N` G + 2.

## `documento buscar` (#61)

| Instalação | Termo | Saída do IES | Req. | Veredito |
| --- | --- | --- | ---: | --- |
| UFRJ | `99079.9017` (fragmento de NUP formatado) | `lista`, 10 linhas, total 73, faixa 1–10 | 3 | **PASS** |
| UFRJ | `0003105` (número SEI exato) | `documento_unico`, ids 5127 / 5126 (os do `documento 0003105`) | 3 | **PASS** |
| PEN3 | `98390.0000` | `lista`, 10 linhas, total 46 (o mesmo do ensaio anterior) | 3 | **PASS** |
| PEN4 | `98490.0000` | `lista`, 10 linhas, total 268 | 3 | **PASS** |
| PEN4 | `0000872` | `documento_unico`, ids 1398 / 1396 (os do `documento 0000872`) | 3 | **PASS** |

## Achados

1. **Processo visitado na lista da pesquisa de processos (FAIL,
   corrigido).** Na PEN4, `processo buscar 98490.0000` saiu
   `html_de_pesquisa_nao_interpretado` (resposta 200 de 133 KB). O HTML
   não é guardado pelo IES, então a resposta ao vivo não foi lida. Causa
   estrutural provável: o link do processo que a conta já visitou vem com
   `class="protocoloNormal processoVisitado" … class="protocoloNormal"`
   (forma gravada na cassette pública `documento-numero-e-busca-pen3`,
   passo 13). A leitura da lista em documentos já aceitava essa forma; a de
   processos só aceitava `class` repetido **idêntico** (UnB) e recusava a
   tag inteira. A conta da PEN4 tinha visitado vários processos `98490.`
   nesta rodada e no ensaio de versões. Correção: a lista de processos
   passa a aceitar `class` repetido quando todas as ocorrências contêm a
   classe procurada (o critério da lista em documentos); uma ocorrência
   sem a classe continua sendo recusada. RED: 1 teste
   (`processo_visitado_com_class_repetido_contendo_a_classe_e_lido`)
   falhando antes da correção, GREEN depois; o replay do acervo de
   pesquisa não mudou. A lista da PEN4 com a correção ao vivo é
   **NOT_RUN**.
   O caso `--pagina 2` foi enviado 61 s depois, na mesma sequência, sem
   ler o resultado anterior. Na prática repetiu o POST da página 1. O IES
   não fez retry, mas foi uma segunda tentativa da mesma lista.
2. **`--versao N --saida` grava `DOC.txt`.** O texto de uma versão
   histórica usa o mesmo nome do texto atual. Numa pasta que já tem o
   texto atual, a leitura gasta G + 2 requisições e sai
   `arquivo_texto_documento_ja_existe`, porque a conferência acontece
   depois de tudo lido. A conferência antecipada não é segura, porque um
   externo TXT também grava `DOC.txt` e o tipo só se sabe depois da
   árvore. Nomear o texto histórico (`DOC.vN.txt`, por exemplo) é decisão
   de contrato, não tomada aqui. Os quatro subcódigos de `DOC.txt` não
   estavam na tabela do contrato Web: foram acrescentados.
3. **UFRJ não acha o NUP gerado no dia.** `99079.901789/2026-19`
   (gerado às 11:09 do mesmo dia, na caixa) saiu `sem_resultado` pela
   Pesquisa, e a lista `99079.9017` começou em `901787`, sem `901788` e
   `901789`. Pela mesma barra, em documentos, a lista também não trouxe o
   `0003109` do dia. Hipótese não verificada: o índice de pesquisa da
   instalação ainda não tinha esses protocolos. O comando já publica isso
   como `sem_resultado`, sem afirmar inexistência.
4. **Bloqueio da borda da PEN3.** Às 21:26 UTC, `documento 0001054`
   recebeu 403 do Cloudflare na terceira requisição (a árvore, depois do
   302 da pesquisa rápida e da tela), sem termo na URL. A mensagem diz
   "processoeletronico.gov.br", a zona da PEN3 e da PEN4, que nesta
   rodada recebiam comandos intercalados (cerca de dois comandos a cada
   22 s na zona). Na PEN4 não houve bloqueio depois de uma pausa de 10 min
   e de passar a ≥ 60 s entre comandos. É mais um indício de que a borda
   bloqueia por ritmo agregado na zona, e não pelo termo. O IES publicou o
   bloqueio tipado com `status_http` 403 (no ensaio de versões o mesmo 403
   no POST saía como `pesquisa_rapida_nao_abriu_o_documento`).

## Não executado

- PEN3: `processo buscar` de controle, documento interno frio e número
  SEI exato em `documento buscar` (já PASS no
  [ensaio anterior](2026-11-04-documento-buscar.md)), parados pelo
  bloqueio.
- Externo TXT ao vivo: os TXT das cassettes (`0001019` na PEN3,
  `0000852` na PEN4) não estão mais nas árvores lidas. Externo na UFRJ: o
  único PDF achado (`0000094`) é de outra unidade e foi recusado de forma
  tipada.
- Lista da PEN4 com a correção do achado 1.

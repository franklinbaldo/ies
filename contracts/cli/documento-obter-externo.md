---
type: Contract
title: Ler documento externo pela Web
description: ies documento DOC lê o arquivo de um documento externo pelo link documento_download_anexo assinado que a árvore publica, por um GET binário com limites próprios, e publica metadados, texto (só tipos de texto) ou o arquivo original em --saida
---

# `ies documento DOC` externo (Refs #60, fatia 3)

Complementa a [leitura Web](documento-obter-web.md): quando o
`Nos[i].src` do nó é `documento_download_anexo` (PDF, TXT, HTML
recebido), o passo 5 deixa de ser recusa e passa a ser um **GET binário**
do link que a árvore publicou. Decisão:
[documento externo](../../docs/decisoes/documento-obter/documento-externo.md).

## Link

Só o link assinado da árvore: `controlador.php?acao=documento_download_anexo`
com exatamente um `id_anexo` decimal, um `infra_hash` de 64 hex e chaves
canônicas (forma das cassettes de PEN1, PEN2 e PEN4: `acao_origem`,
`id_anexo`, `arvore`, `infra_sistema`, `infra_unidade_atual`,
`infra_hash`). O link não traz `id_documento`: a ligação com o documento
é o próprio nó da árvore já conferido (id e número SEI). O IES nunca monta
esse link (ADR 0003). Custo igual ao do interno em qualquer caminho da
leitura Web (índice: 2, 3 ou 4; `--processo NUP`; pesquisa rápida: 4 ou
5): o GET binário toma o lugar do GET do conteúdo.

## Transporte binário

Um caminho próprio do cliente HTTPS único (`transporte::resposta_binaria`),
uma tentativa, sem seguir redirect (3xx é recusa com o status):

- só status 200; os demais usam as recusas de status do transporte;
- `Content-Type` obrigatório e único; o tipo principal tem de estar na
  lista observada: `application/pdf` (UnB, pelo oráculo Todos; ao vivo na
  PEN3 e na PEN4 5.0.4, `inline`, em 04/11/2026), `text/plain`
  (ícone `documento_txt`) e `text/html` (HTML recebido, gravado em PEN1,
  PEN2 e PEN4 como `text/html;;charset=iso-8859-1`; parâmetros vazios são
  tolerados);
- `Content-Length` e `Content-Disposition` são opcionais, mas cada um no
  máximo uma vez; `Content-Disposition` presente tem de ser `inline` ou
  `attachment`, com no máximo um `filename` e um `filename*`;
- limite: `application/pdf` até `IES_LIMITE_ANEXO_BYTES` (padrão
  64 MiB, de 1 a 2 GiB); texto até o menor entre esse limite e 8 MB. O
  `Content-Length` acima do limite recusa antes de ler o corpo; sem ele,
  o corpo (inclusive `chunked`) é limitado **durante** a leitura;
- o corpo é lido em fluxo, em blocos, com SHA-256 calculado na leitura;
  PDF nunca fica inteiro na memória;
- cabeçalho com bytes fora de UTF-8 (nome de arquivo em ISO-8859-1) é
  lido como ISO-8859-1, em vez de derrubar a resposta;
- `text/html` que é a tela de login vira `sessao_encerrada_rode_ies_login`;
  com o contêiner de exceção do SEI (`divInfraExcecao`, `infraExcecao`,
  `infraMensagemErro`), `anexo_recusado_pelo_sei` com a mensagem saneada.

## Saída

O JSON de cada documento externo traz `fonte: "documento_download_anexo"`,
`tipo: null` (o título do HTML recebido é do órgão de origem, não da
árvore) e `anexo`:

| Campo | Conteúdo |
| --- | --- |
| `mime` | tipo principal, em minúsculas |
| `tamanho` | bytes lidos |
| `sha256` | do corpo lido |
| `nome_original` | `filename*` (RFC 5987) ou `filename` do `Content-Disposition`, saneado (sem diretório nem controle, até 255 caracteres); `null` sem o cabeçalho |
| `disposicao` | `inline`, `attachment` ou `null` |

Sem `--saida`: `texto` só para tipos de texto (`text/plain` decodificado
no charset declarado ou UTF-8 estrito, controles trocados por `�`;
`text/html` pelo texto visível do corpo, como o interno); PDF sai com
`texto: null` e só os metadados. Bytes binários nunca vão para a saída
padrão. Sem `--json`, a linha do documento, a linha do anexo e o texto
(se houver).

Com `--saida DIR`: o arquivo **original** vai para `DOC.pdf`, `DOC.txt`
ou `DOC.html` (extensão pelo `Content-Type`; o nome do servidor nunca é
caminho). O corpo é gravado em `DIR/.DOC.ext.parcial-PID` (0600, criado
novo) durante a leitura e ligado ao nome final só depois de todos os
documentos lidos, sem sobrescrever; qualquer falha remove o parcial. O
JSON troca `texto` por `arquivo` e `bytes`.

## Falhas tipadas

| Subcódigo | Fato |
| --- | --- |
| `limite_de_anexo_invalido` (saída 2) | `IES_LIMITE_ANEXO_BYTES` não é inteiro de 1 a 2 GiB |
| `anexo_sem_tipo_de_conteudo` | resposta 200 sem `Content-Type` |
| `tipo_de_anexo_nao_permitido` | tipo principal fora da lista observada |
| `cabecalho_do_anexo_repetido` | `Content-Type`, `Content-Length` ou `Content-Disposition` mais de uma vez |
| `disposicao_do_anexo_fora_da_forma_observada` | `Content-Disposition` sem `inline`/`attachment` ou com nome repetido ou ilegível |
| `anexo_acima_do_limite` | declarado ou lido acima do limite |
| `anexo_vazio` | corpo sem bytes |
| `anexo_recusado_pelo_sei` | HTML com o contêiner de exceção do SEI |
| `diretorio_do_anexo_ausente` / `diretorio_do_anexo_link_simbolico` | `--saida` não é diretório, ou passa por link simbólico |
| `arquivo_do_anexo_ja_existe` | `DOC.ext` já existe (conferido antes do corpo e ao ligar) |
| `escrita_do_anexo_falhou` | o parcial não pôde ser criado, escrito ou ligado |

Status, redirect, rede, charset e login usam os subcódigos do transporte
e de `processo NUP`. O primeiro documento que falha encerra o comando,
como na leitura interna.

## Efeitos

Leitura. Se o GET do anexo marca o documento como visitado não foi
medido.

Validação live: PDF **PASS** na PEN3 e na PEN4 5.0.4 e HTML recebido na
PEN4, frio e quente; TXT e UFRJ **NOT_RUN**
([varredura](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)).

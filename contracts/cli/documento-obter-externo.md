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

O caminho principal usa só o link assinado da árvore: `controlador.php?acao=documento_download_anexo`
com exatamente um `id_anexo` decimal, um `infra_hash` de 64 hex e chaves
canônicas (forma das cassettes de PEN1, PEN2 e PEN4: `acao_origem`,
`id_anexo`, `arvore`, `infra_sistema`, `infra_unidade_atual`,
`infra_hash`). O link não traz `id_documento`: a ligação com o documento
é o próprio nó da árvore já conferido (id e número SEI). O IES nunca monta
esse link (ADR 0003). Custo igual ao do interno em qualquer caminho da
leitura Web (índice: 2, 3 ou 4; `--processo NUP`; pesquisa rápida: 4 ou
5): o GET binário toma o lugar do GET do conteúdo.

Se o `.src` de um externo não cancelado estiver vazio, mas a barra publicar
`documento_alterar_recebido` ou `documento_consultar_recebido`, o IES faz
**um GET adicional** desse cadastro (sem POST), confere os IDs ocultos,
identifica o `id_anexo` em `hdnAnexos` e segue somente o link assinado
`documento_download_anexo` publicado por `objTabelaAnexos.adicionarAcoes`.
Sem vínculo inequívoco permanece a recusa, sem fabricar URL. Esse fallback
foi derivado de `todos/tests/paginas_reais/documento_alterar_recebido.html`
e da [issue Todos #1246](https://github.com/franklinbaldo/todos/issues/1246).
A prova é offline; **live do fallback: NOT_RUN**.

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
- `text/html` que é a tela de login vira `SessaoEncerrada`;
  com o contêiner de exceção do SEI (`divInfraExcecao`, `infraExcecao`,
  `infraMensagemErro`), `ErroAnexo::RecusadoPeloSei` com a mensagem saneada.

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
JSON troca `texto` pelo objeto `arquivo` (`nome`, `bytes`, `sha256` do
anexo, `gravado: true`, `motivo: null`), a mesma forma da foto do
interno.

O externo **não** segue a foto `DOC.vN.html` de #207
([decisão](../../docs/decisoes/documento-obter/saida-idempotente.md)):
não serve rodapé nem publica a lista de versões (observado), então não
há versão para o nome, e `--versao N` é recusado antes do anexo. O
`DOC.txt` daqui é o arquivo original `text/plain`, não um texto
extraído. Um `DOC.ext` já presente continua recusado
(`ErroAnexo::ArquivoJaExiste`). Desde #207, o `DIR` ausente é recusado
antes da rede (`ErroArquivoTexto::DiretorioAusente`); desde 05/11/2026 o
`DIR` por link simbólico é aceito e o modo do `DIR` não é conferido (é
da pessoa); o nome final nunca é seguido.

## Falhas tipadas

| Erro ou `onde` (stderr) | Fato |
| --- | --- |
| `LimiteDoAnexoInvalido` (saída 2) | `IES_LIMITE_ANEXO_BYTES` não é inteiro de 1 a 2 GiB |
| `ErroAnexo::SemTipoDeConteudo` | resposta 200 sem `Content-Type` |
| `ErroAnexo::TipoNaoPermitido` | tipo principal fora da lista observada |
| `ErroAnexo::CabecalhoRepetido` | `Content-Type`, `Content-Length` ou `Content-Disposition` mais de uma vez |
| `ErroAnexo::DisposicaoForaDaFormaObservada` | `Content-Disposition` sem `inline`/`attachment` ou com nome repetido ou ilegível |
| `ErroAnexo::AcimaDoLimite` | declarado ou lido acima do limite |
| `ErroAnexo::Vazio` | corpo sem bytes |
| `ErroAnexo::RecusadoPeloSei` | HTML com o contêiner de exceção do SEI |
| `ErroAnexo::DiretorioAusente` | `--saida` não é diretório (link no caminho é aceito; o modo do `DIR` não é conferido) |
| `ErroAnexo::ArquivoJaExiste` | `DOC.ext` já existe (conferido antes do corpo e ao ligar) |
| `ErroAnexo::EscritaFalhou` | o parcial não pôde ser criado, escrito ou ligado |

Status, redirect, rede, charset e login usam os subcódigos do transporte
e de `processo NUP`. O primeiro documento que falha encerra o comando,
como na leitura interna.

## Efeitos

Leitura. Se o GET do anexo marca o documento como visitado não foi
medido.

Live: PDF **PASS** na UnB 4.0.9 em 04/11/2026, com e sem `--saida`
(2 requisições pelo link guardado, SHA-256 do arquivo igual ao publicado;
[varredura UnB e GO](../../docs/validacao-live/2026-11-04-varredura-unb-go.md)).

Validação live: PDF **PASS** na PEN3 e na PEN4 5.0.4 e HTML recebido na
PEN4, frio e quente; TXT **NOT_RUN**; UFRJ só recusa tipada (PDF de outra unidade)
([varredura](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)).

## Cache do original (#236)

Por padrão os bytes do GET binário são gravados em cache privado
`ies/<host>/<DOC>/x-<sha256[:16]>.<pdf|txt|html>`, sem processo
no caminho, com escrita em fluxo e nenhuma requisição adicional.
Com `--saida`, o arquivo opcional é derivado do cache por ligação
física ou cópia, sem substituir conteúdo divergente.
A extensão vem do MIME e nunca do nome do servidor.
`--sem-cache` ou `IES_CACHE_DOCUMENTOS=0` preservam a leitura
sem cache, com `--saida` explícita ainda disponível.

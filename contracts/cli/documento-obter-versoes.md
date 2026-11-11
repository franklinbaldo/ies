---
type: Contract
title: Ler versão de documento e gravar a foto DOC.vN.html
description: ies documento DOC --versao N lê a versão N pela lista de versões que a barra do documento publica na árvore; versao_atual sai do rodapé servido ou da lista lida, nunca de um GET só para ela; com --saida, grava só DOC.vN.html, o HTML servido da versão lida, de forma idempotente (#207)
---

# Versões e foto de `ies documento DOC` (Refs #88, #207)

Complementa a [leitura Web](documento-obter-web.md). A árvore é aberta
pelo caminho que a leitura Web escolher (`--processo NUP`, índice ou
pesquisa rápida, Refs #60 fatia 2); este contrato trata do que vem depois
dela, e a barra lida é a do `DOCUMENTO` nessa árvore.

```text
ies documento DOC [DOC ...] [--versao N] [--saida DIR] [--json] [--unidade SIGLA]
```

## Onde a lista está publicada

A barra do nó `DOCUMENTO` (`Nos[i].acoes`, o literal inteiro de uma
linha) publica o link assinado
`controlador.php?acao=documento_versao_listar&…&id_procedimento=P&id_documento=D&…&infra_hash=…`
("Versões do Documento"). Medido no acervo público: as sete instalações
(UnB, UFRJ, GO, PEN1 a PEN4) gravaram a lista seguindo esse link da
barra. O link sem assinatura derruba a sessão (ADR 0003): o IES só segue
o link da barra, conferido (`acao` única, `id_documento` igual ao do nó,
`id_procedimento` igual ao do processo, `infra_hash` de 64 hex). Documento
externo, de outra unidade ou sem permissão não traz o link.

A mesma barra declara a edição (`editarConteudo('N')` ou `'S'`); a
foto não depende dela. O IES não segue nenhum desses links e nunca abre
o editor (`editor_montar`).

O conteúdo (`documento_visualizar`, atual ou da versão N) serve abaixo do
texto um rodapé gerado pelo SEI, fora do conteúdo editável:
`<div unselectable="on">…Criado por <a>U</a>, versão N por <a>U</a> em DATA.</div>`.
Visto nas 91 respostas HTML do acervo (UnB, PEN1 a PEN4; UFRJ e GO não
gravaram `documento_visualizar`) e ao vivo na PEN4 5.0.4, onde a versão 1
lida pela lista declarou `versão 1` e a atual `versão 2`, e depois
(04/11/2026) na PEN3 5.0.4 e na UFRJ 4.1.5 ([varredura](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)). Só um rodapé
nessa forma exata conta; nenhum ou dois é "não servido".

A tela da lista (`table.infraTable` com legenda única
`Lista de Versões (N registros):`) traz, por linha, versão, usuário,
unidade, data e, em Ações, **um** link `documento_visualizar` assinado
com `versao=N`, o `id_documento` do nó (servido duas vezes, com o mesmo
valor, nas sete instalações) e o `id_procedimento`. Outras âncoras da
célula (`href="#ID-N"` do "Recuperar Versão", cujo link fica no
`onclick`) não são seguidas. Total declarado diferente das linhas lidas
é recusa.

## Caminhos e requisições

Sem retry e sem seguir redirect. `G` = as requisições até a árvore pelo
caminho escolhido: pela pesquisa rápida, 3 com o `action` guardado e 4
sem ele (Controle, POST, tela, árvore); pelo link guardado da árvore, 1;
os demais caminhos, como na [leitura Web](documento-obter-web.md).

| Pedido | Requisições | O que se lê |
| --- | --- | --- |
| sem `--versao` (com ou sem `--saida`) | G + 1 | o conteúdo atual (`Nos[i].src`); a versão vem do rodapé |
| `--saida DIR` sem rodapé e barra com a lista | G + 2 | conteúdo atual e a lista (para saber N) |
| `--saida DIR` sem rodapé e sem a lista | G + 1 | recusa `ListaDeVersoesNaoPublicadaNaArvore`, nada gravado |
| `--versao N` (com ou sem `--saida`) | G + 2 | a lista e o `documento_visualizar` da linha N; o conteúdo atual **não** é pedido |

Decisão de custo (proprietário, 04/11/2026): `versao_atual` é `null`
salvo quando pedida (`--versao`) ou já servida na tela lida. O rodapé é
servido na tela do conteúdo, então sai sem GET a mais; a lista (1 GET)
só é pedida por `--versao N` ou pela foto quando falta o rodapé.

## Saída

Cada documento do JSON da leitura Web ganha:

- `versao`: a versão do texto publicado (N de `--versao N` ou a do
  rodapé), senão `null`;
- `versao_atual`: a maior versão da lista, quando lida, ou a do rodapé
  do conteúdo atual; senão `null`;
- `versao_atual_motivo`: `null` quando publicada; senão
  `nao_servida_no_conteudo` (sem rodapé, lista não pedida);
- com `--saida`, `arquivo` (no lugar de `texto`, como na leitura Web):
  `{"nome": "DOC.vN.html", "bytes": B, "sha256": "…", "gravado": true,
  "motivo": null}`, ou `"gravado": false, "motivo": "igual"` quando a
  foto já existia com o mesmo SHA-256.

Sem `--saida`, `texto` é o da versão lida. Nunca URL, `infra_hash`,
cookie ou HTML no terminal. Os campos `copia` e `copia_motivo` de #88
saíram com a cópia de trabalho.

## Foto `DOC.vN.html` com `--saida` (#207)

Decisão do proprietário (05/11/2026,
[decisão](../../docs/decisoes/documento-obter/saida-idempotente.md)),
que **substitui** `DOC.txt`/`DOC.vN.txt` (#204) e a cópia de trabalho
com base e manifesto ocultos (#88). Uma versão fechada não muda: o nome
carrega a identidade inteira e o arquivo é a foto fiel da versão.

- **Um arquivo por versão.** `--saida DIR` grava só `DOC.vN.html`, o
  HTML que o SEI serviu da versão N lida (atual ou histórica, assinada
  ou não), em UTF-8. Nenhum `.txt`, nenhum arquivo oculto. Quem quiser
  texto converte o HTML. A saída sem `--saida` não muda.
- **N.** Da versão pedida (`--versao N`) ou do rodapé servido; sem
  rodapé, da lista (G + 2); sem lista publicada, recusa tipada, sem
  gravar com nome incerto.
- **Idempotente.** Ausente: grava. Presente com o mesmo SHA-256: não
  grava, sai 0 e diz `gravado: false`. Presente com outro conteúdo:
  `ErroCopia::FotoDivergente`. O lote inteiro é conferido antes da primeira
  gravação: uma divergência não grava nenhuma foto.
- **Antes da rede.** `DIR` tem de existir e ser diretório (link no
  caminho, inclusive o próprio `DIR`, é aceito; o modo do `DIR` é da
  pessoa e não é conferido) (0 requisições). Com `--versao N` o nome já é
  conhecido: cada `DOC.vN.html` presente tem de ser legível, privado e
  coerente com o nome (título do mesmo número SEI; rodapé, se houver, da
  versão N), senão `ErroCopia::FotoDivergente` com 0 requisições. Os bytes só
  se comparam depois da leitura.
- **Gravação.** Parcial `0600` criado novo em `DIR`, `fsync` e
  `hard_link` para o nome final (falha se o nome surgiu: nunca
  sobrescreve), sem seguir link. HTML com `infra_hash` não vai a arquivo
  (`ConteudoComLinkAssinado`).
- **Sem registro.** Nada vai para o índice local: o arquivo é a única
  memória da foto. Quem confere que `DOC.vN.html` ainda é o que o SEI
  serve é a releitura (`ErroCopia::FotoDivergente`, acima) e, no futuro, o
  salvar remoto (#89), que relê a versão N. A prévia de
  [`documento salvar`](documento-salvar.md) é offline e não a verifica.
- **Edição (#89).** O agente cria `DOC.v(N+1).html` a partir de
  `DOC.vN.html`; a prévia local compara o par. A escrita remota segue
  pendente.

## Falhas tipadas

| Erro (stderr) | Fato |
| --- | --- |
| `ErroVersao::VersaoDocumentoInvalida` | `--versao 0` (antes de qualquer rede) |
| `ListaDeVersoesNaoPublicadaNaArvore` | `--versao N` e a barra do documento não publica a lista |
| `VersaoPedidaAusenteNoHistorico` | N não está entre as linhas da lista |
| `VersaoServidaDiferenteDaPedida` | o rodapé da versão servida declara outra versão |
| `ErroDocumentoObter::BarraDoDocumento(ErroBarraDoDocumento::{BarrasRepetidasNaArvore, BarraDoDocumentoForaDaFormaObservada, LinkDasVersoesRepetidoNaBarra, LinkDasVersoesNaoReconhecido, EnderecoDasVersoesDeOutroDocumento})` | barra repetida, fora da forma, link repetido, de outro documento ou processo |
| `ErroVersao::HistoricoDocumentoAusente`, `ErroVersao::ListaIncompleta`, `ErroVersao::ColunasHistoricoInesperadas`, `ErroVersao::LinhaIncompleta`, `ErroVersao::LinkVersaoAusenteOuAmbiguo`, `ErroVersao::LinkInseguro`, `ErroVersao::DocumentoDivergente`, `ErroVersao::VersaoDocumentoDuplicada`, `ErroVersao::LegendasHistoricoVersoesRepetidas` | a lista servida fora da forma observada |
| `ErroDocumentoObter::TituloDeOutroDocumento` | a versão servida é de outro número SEI |
| `ErroDocumentoObter::DocumentoCancelado`, `ConteudoNaoPublicadoNaArvore` | como na leitura Web, antes da lista |
| `ListaDeVersoesNaoPublicadaNaArvore` | `--saida`, sem rodapé, e a barra não publica a lista |
| `ConteudoComLinkAssinado` | `--saida` e o HTML servido traz `infra_hash` |
| `ErroCopia::FotoDivergente` | `DOC.vN.html` presente com outro conteúdo, ou (antes da rede, com `--versao N`) com título ou rodapé de outro documento ou versão |
| `ErroArquivoTexto::DiretorioAusente`, `ErroCopia::DestinoInseguro` | `DIR` ausente ou não diretório (antes da rede) |
| `ErroCopia::LinkSimbolico`, `ErroCopia::PermissoesInseguras`, `ErroCopia::LeituraFalhou`, `ErroCopia::ConsultaDaFotoFalhou(ErrorKind)`, `ErroCopia::GravacaoFalhou`, `ErroCopia::GravacaoDaFotoRecusada(FalhaDaGravacao::…)` | a foto presente não pôde ser lida com segurança, ou a gravação falhou (nada publicado) |

## Documento externo

O externo (`.src` `documento_download_anexo`) é lido pelo
[GET binário](documento-obter-externo.md) (fatia 3 do #60). O arquivo
não serve rodapé e não há versão para o nome: sem `--versao`,
`versao: null`, `versao_atual: null` com `nao_servida_no_conteudo`; a
barra não é lida. Com `--saida`, grava o arquivo original `DOC.ext` como
antes (sem sobrescrever, [externo](documento-obter-externo.md)), não uma
foto. Com `--versao N`, a barra do nó (0 GET) é lida antes
do anexo: sem a lista (o observado) é `ListaDeVersoesNaoPublicadaNaArvore`,
em G requisições, sem o GET do anexo; com a lista (não observado num
externo), `barra_do_documento_fora_da_forma_observada`.

## Efeitos

Leitura. A lista e a versão são GET de telas publicadas; se marcam algo
como visitado não foi medido. Nenhuma escrita no SEI, nenhum editor,
nenhuma recuperação de versão. Localmente, só a foto em `DIR`.

Live na UnB 4.0.9 (04/11/2026, [varredura UnB e GO](../../docs/validacao-live/2026-11-04-varredura-unb-go.md)): `--versao 1` **PASS**
(versão 1 de 3, 4 requisições); processo concluído ou aberto só em outra
unidade não publica a lista na barra (`ListaDeVersoesNaoPublicadaNaArvore`).

Validação live: `--versao N` **PASS** na PEN4, na PEN3 e na UFRJ 4.1.5
(G + 2) ([varredura](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)).
Foto idempotente (#207) **PASS** na UnB 4.0.9 em 05/11/2026: a primeira
leitura grava `DOC.v3.html` (G + 1), a segunda acha o mesmo SHA-256 e
não grava; `--versao 1` recusou tipado porque a sessão estava noutra
unidade ([ensaio](../../docs/validacao-live/2026-11-05-documento-saida-idempotente.md)).
A foto de versão histórica e a foto pela lista seguem só com testes
sintéticos.

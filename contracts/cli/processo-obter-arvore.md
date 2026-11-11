---
type: Contract
title: Abrir processo pela árvore
description: ies processo NUP abre o processo com o mínimo de requisições, lê o primeiro nível da árvore, os processos anexados, os andamentos, as propriedades e os marcadores e aprende no índice local
---

# `ies processo NUP` pela árvore (Refs #59, fatias 1 a 4)

Complementa o [contrato de `processo.obter`](processo-obter.md).
`ies processo NUP [--sem-documentos] [--sem-andamentos |
--andamentos-completos] [--sem-propriedades] [--sem-marcadores] [--json] [--unidade SIGLA]` usa a sessão de
`ies login`, abre o processo e publica a identidade, os documentos da
árvore, inclusive os de pastas ([pastas](processo-obter-pastas.md)), os
processos anexados, os andamentos
([seção própria](processo-obter-andamentos.md)) e as propriedades
([seção própria](processo-obter-propriedades.md)) e os marcadores, sempre
como lista ([seção própria](processo-obter-marcadores.md)), onde o
processo está aberto e a natureza de cada documento, sem requisição a
mais ([seção própria](processo-obter-situacao.md)). `--resumo` foi
aposentado em 04/11/2026 ([contrato](processo-obter.md)).

## Resolução (ordem aprovada pelo proprietário)

Antes de qualquer requisição, o IES lê o cookie e calcula a geração da
sessão (SHA-256 hexadecimal do cookie sem espaços nas pontas, a mesma
função do marcador de `ies login`) e o id da unidade (`infra_unidade_atual`
único e decimal na URL privada do Controle). Os links do
[índice local](../../docs/arquitetura/0003-indice-local.md) são buscados
por essa geração e por essa unidade. Cada passo é tentado só se o
anterior não se aplica ou falhou por palpite do índice:

1. **Link da árvore no índice** (`resolvido_por: "indice_link"`): o NUP
   tem id conhecido e há link `procedimento_visualizar` guardado para ele
   nesta geração e unidade. Um GET. A árvore não publica a unidade, então
   `unidade` sai `null`; com `--unidade` o passo é pulado.
1b. **Link `procedimento_trabalhar` no índice** (`"indice_trabalhar"`):
   o NUP tem id conhecido e há link `procedimento_trabalhar` guardado para
   esse id nesta geração e unidade (colhido de uma linha do Controle por
   qualquer comando que o leu, ou aprendido numa abertura anterior). GET
   dele → tela do processo (unidade conferida com `--unidade`) → GET da
   árvore. Roda também com `--unidade`.
2. **Entrada por id** (`"indice_entrada"`): id conhecido e
   `entrada_por_id` diferente de recusada. GET de
   `controlador.php?acao=procedimento_trabalhar&id_procedimento=N` (a
   única URL sem assinatura do IES) → 302 para `procedimento_trabalhar`
   assinado com o **mesmo** id e sem `acao_origem` → GET → GET da árvore.
   O aceite é registrado. A recusa característica (302 para `login.php`
   com `msg` igual a `Link sem assinatura`) é registrada e encerra o
   comando com `ErroProcessoObter::EntradaPorIdRecusada`: a recusa derruba
   a sessão; não há relogin automático nem nova tentativa.
3. **Pesquisa rápida guardada** (`"pesquisa_rapida"`): link
   `protocolo_pesquisa_rapida` guardado (objeto vazio). POST de
   `txtPesquisaRapida=NUP` para ele (`Referer` = URL do Controle; NUP só
   ASCII, porque o charset da tela não foi guardado) → 302 → GET → GET.
4. **Controle** (`"caixa"` ou `"pesquisa_rapida"`): GET do Controle (unidade
   conferida com `--unidade`). (a) A linha `tr#P{id}` cujo link de
   `procedimento_trabalhar` tem texto **igual** ao NUP → GET → GET; NUP
   em duas linhas ou dois links na linha é recusa. (b) Senão, o POST do
   `form#frmProtocoloPesquisaRapida` (método post, `action` assinado, um
   controle `txtPesquisaRapida`, envio por script, sem botão) → 302 → GET
   → GET. A URL do `action` é guardada no índice.

O 302 da pesquisa rápida observado em GO
(`processo-andamento-go`, `processo-nup-inexistente-go`) aponta para
`procedimento_trabalhar` com `acao_origem=protocolo_pesquisa_rapida` e o
id em `id_protocolo`; `infra_unidade_atual` e `infra_sistema` têm de ser
iguais aos do `action`. Qualquer outra resposta à pesquisa rápida é
`PesquisaRapidaSemUrlDeTrabalho` (HTML em vez do 302); um status HTTP sobe
como `Transporte(Status(..))` e a validação da Infra como
`Formulario(ValidacaoRecusadaPeloSei(mensagem))`; a página de resultado
nunca é lida.
`ies processo NUP` **nunca** usa a pesquisa avançada: em GO e na PEN1 ela
não achou NUPs que a pesquisa rápida abriu (medição do agente pai,
03/11/2026).

**Palpites do índice.** No passo 1, link que responde login, redirect ou
outro status, e árvore de outro NUP ou id, apagam o link (e, na
divergência de identidade, a linha do processo) e o comando segue. No
passo 1b, login, redirect, outro status, página que não é a tela do
processo ou tela de outro processo apagam o link e o comando segue;
árvore de outro NUP apaga também a linha do processo. No
passo 2, 302 para outro id ou árvore de outro NUP apagam a linha do
processo e o comando segue. No passo 4, árvore de outro NUP é erro e
apaga a linha do processo (a colheita do mesmo GET a teria gravado). No passo 3, login apaga o link e segue.
Fora de palpite, login (200 ou 302 sem a mensagem de recusa) é
`SessaoEncerrada`. Falha do banco nunca derruba o comando.

**Aprendizado** (só do que o comando já leu): `registrar_processo(NUP, id)`
e, de cada anexado da árvore conferida, `registrar_processo(a5, a1)`;
aberto um anexado, o par do pai e o link `procedimento_trabalhar` dele
(objeto = id do pai, que o passo 1b usa); de cada documento movido com
destino conferido, o par e o link `procedimento_trabalhar` do destino
(objeto = id do destino);
os links `procedimento_trabalhar` e `procedimento_visualizar` (objeto = id),
o de "Abrir todas as Pastas" (objeto = `<id>:abrir_pastas`, só depois da
árvore aberta conferida) e o `action` da pesquisa rápida da tela do
processo. O GET do Controle do passo 4 passa pela
[colheita](../../docs/arquitetura/0003-indice-local.md)
(linhas, link da pesquisa e `action` da pesquisa rápida), sem requisição
a mais.

## Orçamento por caminho

| Caminho | Requisições (até a árvore) |
| --- | --- |
| 1. link da árvore | 1 GET |
| 1b. link `procedimento_trabalhar` | 2 GET |
| 2. entrada por id | 3 GET |
| 3. pesquisa rápida guardada | 1 POST + 2 GET |
| 4a. linha do Controle | 3 GET |
| 4b. pesquisa rápida do Controle | 3 GET + 1 POST |

Assim, `ies processo NUP` de um processo que um comando anterior viu no
Controle, na mesma sessão, custa 3 requisições com andamentos e 2 com
`--sem-andamentos`.
Os andamentos somam 1 GET (padrão) ou 1 GET + 1 POST
(`--andamentos-completos`), e nada com `--sem-andamentos` ou sem link
publicado na árvore ([orçamento](processo-obter-andamentos.md)). Pasta
fechada (`AGUARDE`) soma 1 GET; o link guardado de "Abrir todas as Pastas"
faz o caminho 1 custar 1 ([pastas](processo-obter-pastas.md)).
As propriedades e os marcadores somam 1 GET cada (nada com
`--sem-propriedades`/`--sem-marcadores` ou sem link na barra;
[propriedades](processo-obter-propriedades.md),
[marcadores](processo-obter-marcadores.md)).

Um palpite que falha soma suas requisições às do passo seguinte
(`requisicoes` publica o total). Sem retry, sem redirect seguido pelo
cliente.

## Tela do processo e árvore

A resposta 200 de `procedimento_trabalhar` publica `#lnkInfraUnidade`
(par móvel/desktop aceito) e exatamente um `iframe#ifrArvore` cujo `src`
relativo tem uma `acao=procedimento_visualizar`, `id_procedimento` igual
ao aberto e um `infra_hash` de 64 hex. A árvore é lida das linhas inteiras
`Nos[i] = new infraArvoreNo(a0,…,a15);` por um leitor de literais JS
escrito à mão (aspas duplas ou simples com escapes `\\ \" \' \/ \n \r \t`,
`null`, `true`, `false`, inteiros), sem regex e sem executar JavaScript.
Qualquer linha com `new infraArvoreNo(` fora dessa forma é recusa. Índices
consecutivos desde 0, 16 argumentos, um único `PROCESSO` (a1 id decimal,
a2 `null`, a5 NUP, a6 tipo, a15 igual a a5), `DOCUMENTO` com a2 igual ao
id do processo ou a uma pasta; `PASTA` e `AGUARDE` (15 argumentos) seguem
o [contrato das pastas](processo-obter-pastas.md); `PROCESSO_ANEXADO`
e `DOCUMENTO_MOVIDO` seguem as seções abaixo; outro tipo é recusa.
O NUP do nó (a5) é comparado **byte a byte** com o pedido. Estrutura em
[árvore do processo](../../docs/como-o-sei-funciona/arvore-do-processo.md).

**Processos anexados.** Forma observada na UnB 4.0.9 (03/11/2026), a
única aceita: `PROCESSO_ANEXADO` com 16 argumentos, a1 id decimal do
anexado, a2 igual ao id do processo, a3 texto que começa por
`controlador.php?` com uma `acao=arvore_visualizar`, um `id_procedimento`
igual a a2 e um `id_procedimento_anexado` igual a a1, a5 NUP do anexado
(não vazio), a6 tipo, a12 a a14 `null` e a15 igual a a5. Id ou NUP
iguais aos do processo, ou repetidos entre anexados, são recusa. A barra
do anexado (`.acoes`) não é lida. A árvore **não** traz os documentos do
anexado (na captura, todos os documentos eram filhos do processo); um
`DOCUMENTO`, `PASTA` ou `PROCESSO_ANEXADO` com pai num anexado nunca foi
observado e é recusa, nunca palpite. Para ler o anexado:
`ies processo <NUP do anexado>`, que o índice já resolve pelo id aprendido.

**Processo aberto que é anexado.** A árvore do próprio anexado tem raiz
`PROCESSO` comum; o `.html` da raiz (literal JS, lido pelo mesmo leitor de
literais, na linha própria ou seguido da declaração seguinte) traz o
vínculo. As tags `<a>` do `.html` são lidas pelo tokenizador de atributos;
é candidata a que tem `href` com chave `id_procedimento_anexado`. Uma
candidata é o vínculo: `href` relativo `controlador.php?` conferido como
os outros links assinados (uma `acao=procedimento_trabalhar`, um
`infra_hash` de 64 hex, um `id_procedimento` decimal diferente do aberto),
um `id_procedimento_anexado` igual ao aberto e texto não vazio, sem
espaço nas pontas, entidade ou tag: o NUP do pai. Nenhuma candidata:
`anexado: false`. A frase em volta e o ícone não classificam nada.

**Documentos movidos.** `DOCUMENTO_MOVIDO` segue o
[contrato dos movidos](processo-obter-movidos.md): fora de `documentos`,
com o destino do link do nó e `presente_entre_os_documentos`.

```json
{"numero": "NUP", "id_procedimento": "N", "tipo": "a6 do nó PROCESSO",
 "unidade": "SIGLA ou null", "resolvido_por": "caixa",
 "requisicoes": 4,
 "anexado": false, "anexado_a": null,
 "anexados": [{"numero": "a5", "id_procedimento": "a1", "tipo": "a6"}],
 "aberto_em": [{"sigla": "…", "nome": "…", "atribuido_a": {"sigla": "…", "nome": "…"}}],
 "sobrestado_em": null,
 "documentos": [{"id_documento": "D", "rotulo": "a5", "numero_sei": "a15 ou null",
                 "pasta": "PASTA1 ou null", "icone": "documento_interno",
                 "cancelado": false, "motivo_do_cancelamento": null}],
 "documentos_movidos": [{"id": "a1 (n-m)", "rotulo": "a5", "numero_sei": "a15 ou null",
                         "destino": {"numero": "NUP", "id_procedimento": "N"},
                         "presente_entre_os_documentos": false}],
 "pastas": [{"id": "PASTA1", "rotulo": "I", "total": 20, "verificacao": "por_identidade"}],
 "verificacao_das_pastas": "por_identidade, mista, por_contagem ou null",
 "andamentos": {"tipo": "resumido", "total_declarado": 3,
                "faixa_observada": {"inicio": 1, "fim": 3},
                "itens": [{"ocorrido_em": "…", "unidade": "…", "usuario": "… ou null", "descricao": "…"}]},
 "propriedades": {"tipo": "…", "especificacao": "…", "interessados": [],
                  "assuntos": [{"codigo": "… ou null", "descricao": "…"}],
                  "observacoes": "", "nivel_de_acesso": "publico",
                  "hipotese_legal": null, "data_de_geracao": "dd/mm/aaaa",
                  "fonte": "procedimento_alterar"},
 "marcadores": [{"id": "…", "nome": "…", "cor": "…", "texto": "… ou null",
                 "usuario": "… ou null", "data_hora": "dd/mm/aaaa hh:mm"}],
 "marcadores_total_declarado": 1,
 "marcadores_faixa_observada": {"inicio": 1, "fim": 1},
 "nao_lidos": ["documentos_dos_anexados"]}
```

`anexados` sai **sempre**, na ordem dos nós (`[]` sem anexado), também
com `--sem-documentos`: vem da mesma árvore, sem custo, e é identidade do
processo, não documento. Com algum anexado, `nao_lidos` termina em
`documentos_dos_anexados`; o texto humano lista cada um com a dica
`ies processo <NUP>`. `anexado` (`true` com o vínculo) e `anexado_a`
(`{"numero", "id_procedimento"}` do pai, ou `null`) também saem sempre;
o texto humano diz `Anexado ao processo <NUP> (id N): ies processo <NUP>`.

As formas de `andamentos` (objeto, `null` com motivo, ausente) estão em
[andamentos](processo-obter-andamentos.md); as de `propriedades`, em
[propriedades](processo-obter-propriedades.md); as de `marcadores`, em
[marcadores](processo-obter-marcadores.md); as de `aberto_em` e dos
campos de natureza, em [situação](processo-obter-situacao.md).
`nao_lidos` é, na ordem: `aberto_em` (se não lido), `andamentos` (se
não lidos), `marcadores` (se não lidos), `propriedades` (se não lidas)
e `documentos_dos_anexados`.

`documentos` segue a ordem dos nós; com pastas, só sai depois de cada
pasta conferida pela identidade dos protocolos declarados. A dica dos documentos (a6) sai em `dica` quando
difere do rótulo, e o cancelamento é lido dela
([situação](processo-obter-situacao.md)). `documentos_movidos`: [movidos](processo-obter-movidos.md).
`--sem-documentos` tira as
chaves `documentos`, `documentos_movidos`, `pastas` e `verificacao_das_pastas*` (a árvore continua lida: é ela que prova a identidade).
`--sem-marcadores` não pede a tela de marcadores; `--sem-propriedades`
não pede a tela de propriedades.
`--sem-andamentos` com `--andamentos-completos` sai com 2
(`sem_andamentos_incompativel_com_andamentos_completos`).

## Falhas (saída 1)

Fluxo: `ErroProcessoObter::EntradaPorIdRecusada`,
`SessaoEncerrada`,
`RespostaDaEntradaPorIdNaoReconhecida`,
`PesquisaRapidaSemUrlDeTrabalho`, `redirecionamento_nao_reconhecido`,
`processo_repetido_no_controle`, `links_do_processo_repetidos_no_controle`,
`LinkDoProcessoNoControleNaoReconhecido`,
`CharsetDoControleNaoSuportado`. Os 302 conferidos usam os subcódigos
de [processo buscar](processo-buscar.md) e mais
`AcaoDeOrigemInesperadaNoRedirecionamento` e
`IdProcedimentoDoRedirecionamentoDiferenteDoPedido`; o formulário,
os do primitivo de formulário; Controle e transporte, os de sempre.

Tela: `arvore_ausente_na_tela_do_processo`,
`arvores_repetidas_na_tela_do_processo`,
`endereco_da_arvore_nao_reconhecido`,
`EnderecoDaArvoreDeOutroProcesso`,
`html_da_tela_do_processo_nao_interpretado`,
`identificacao_da_unidade_ausente_na_tela_do_processo`,
`UnidadeDaTelaDiferenteDaSolicitada`.

Árvore: `declaracao_de_no_fora_da_forma_observada`,
`ArgumentoDeNoNaoLiteral`, `TextoDeNoSemFechamento`,
`escape_de_no_nao_reconhecido`,
`QuantidadeDeArgumentosDoNoDiferenteDaObservada`,
`ArgumentoDeNoComTipoDiferenteDoObservado`,
`indice_de_no_fora_da_sequencia`, `TipoDeNoDaArvoreNaoSuportado`,
`NoDoProcessoAusenteNaArvore`, `nos_do_processo_repetidos_na_arvore`,
`NoDoProcessoComPai`, `NupDoNoDoProcessoInconsistente`,
`IdDeNoNaoDecimal`, `DocumentoComPaiDiferenteDoProcesso`,
`documento_repetido_na_arvore`,
`IdentidadeDoProcessoDiferenteDaPedida`,
`IdDoProcessoNaArvoreDiferenteDoAberto`.

Natureza do `DOCUMENTO` (a6 a a9): `IconeDoDocumentoForaDaFormaObservada`
quando a7, a8 e a9 não são o mesmo `svg/<radical>.svg` observado;
`IconeDeCancelamentoSemMotivoNaDica` quando o ícone é `documento_cancelado`
sem a dica `Documento Cancelado: <motivo>`; e
`DicaDeCancelamentoSemIconeDeCancelado` quando a dica traz o cancelamento
com outro ícone. Ícone e cancelamento não têm estado "não observado": a
árvore inteira é recusada.

Anexados: `processo_anexado_fora_da_forma_observada`,
`nup_do_no_do_processo_anexado_inconsistente`,
`processo_anexado_com_pai_diferente_do_processo`,
`processo_anexado_repetido_na_arvore`,
`processo_anexado_igual_ao_processo`, `no_com_pai_no_processo_anexado`,
`vinculo_de_anexacao_fora_da_forma_observada`,
`vinculos_de_anexacao_repetidos` (duas candidatas ou dois `.html` da raiz),
`html_do_no_do_processo_fora_da_forma_observada`.

Movidos: os do [contrato dos movidos](processo-obter-movidos.md).

Andamentos: os subcódigos da [seção própria](processo-obter-andamentos.md).
Propriedades: os do [contrato das propriedades](processo-obter-propriedades.md).
Pastas: os do [contrato das pastas](processo-obter-pastas.md).

Validação live da fatia 1: **PASS** na UnB, em GO e na PEN1 ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-arvore.md));
frio (5 req.) e quente (4) com andamentos, propriedades, marcadores, `aberto_em` e ícones **PASS** na UFRJ 4.1.5, PEN3 e PEN4 ([varredura](../../docs/validacao-live/2026-11-04-varredura-ufrj-pen3-pen4.md)).
Fatia 2 (andamentos): **PASS** ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-andamentos.md)).
Fatia 3 (pastas): **PASS** na UnB, frio, quente e sessão nova
([relatório](../../docs/validacao-live/2026-11-03-processo-obter-pastas.md)).
Fatia 4 (colheita do Controle e passo 1b): **PASS** na UnB e na PEN1
([relatório](../../docs/validacao-live/2026-11-03-colheita-do-controle.md)).
Documentos movidos: **PASS** na PEN1 ([relatório](../../docs/validacao-live/2026-11-04-processo-obter-movidos.md)).
Situação e natureza: **PASS** na UnB ([relatório](../../docs/validacao-live/2026-11-04-processo-obter-situacao.md)).
Anexados: **PASS** na UnB nos dois sentidos ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-anexados.md)).
Replay offline do acervo público: 2.066 de 2.073 árvores lidas nas sete
instalações; recusas e achados no [relatório](../../docs/validacao-replay/2026-11-04-processo-obter-acervo.md).
Decisão em [processo.obter](../../docs/decisoes/processo-obter.md).

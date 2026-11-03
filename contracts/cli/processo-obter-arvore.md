---
type: Contract
title: Abrir processo pela árvore
description: ies processo NUP sem --resumo abre o processo com o mínimo de requisições, lê o primeiro nível da árvore, os processos anexados e os andamentos e aprende no índice local
---

# `ies processo NUP` pela árvore (Refs #59, fatias 1 a 4)

Complementa o [contrato de `processo.obter`](processo-obter.md). Sem
`--resumo`, `ies processo NUP [--sem-documentos] [--sem-andamentos |
--andamentos-completos] [--json] [--unidade SIGLA]` usa a sessão de
`ies login`, abre o processo e publica a identidade, os documentos da
árvore, inclusive os de pastas ([pastas](processo-obter-pastas.md)), os
processos anexados e os andamentos
([seção própria](processo-obter-andamentos.md)). Marcadores e
propriedades não são lidos. `--resumo` não muda.

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
   comando com `entrada_por_id_recusada_rode_ies_login`: a recusa derruba
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
`pesquisa_rapida_sem_url_de_trabalho`, com `status_http` e a mensagem da
validação da Infra quando houver; a página de resultado nunca é lida.
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
`sessao_encerrada_rode_ies_login`. Falha do banco nunca derruba o comando.

**Aprendizado** (só do que o comando já leu): `registrar_processo(NUP, id)`
e, de cada anexado da árvore conferida, `registrar_processo(a5, a1)`;
aberto um anexado, o par do pai e o link `procedimento_trabalhar` dele
(objeto = id do pai, que o passo 1b usa);
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
segue a seção abaixo; outro tipo é recusa.
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

**Efeito declarado.** Em GO, o GET da árvore marcou como visualizado um
processo recebido não visualizado (o de `procedimento_trabalhar`, sozinho,
não). O IES não restaura esse estado. Lançamento de andamento: não medido.

## Saída

JSON (sem URL, `infra_hash`, cookie ou HTML; `null` é "não observado"):

```json
{"numero": "NUP", "id_procedimento": "N", "tipo": "a6 do nó PROCESSO",
 "unidade": "SIGLA ou null", "resolvido_por": "caixa",
 "requisicoes": 4,
 "anexado": false, "anexado_a": null,
 "anexados": [{"numero": "a5", "id_procedimento": "a1", "tipo": "a6"}],
 "documentos": [{"id_documento": "D", "rotulo": "a5", "numero_sei": "a15 ou null",
                 "pasta": "PASTA1 ou null"}],
 "pastas": [{"id": "PASTA1", "rotulo": "I", "total": 20, "verificacao": "por_identidade"}],
 "verificacao_das_pastas": "por_identidade, mista, por_contagem ou null",
 "andamentos": {"tipo": "resumido", "total_declarado": 3,
                "faixa_observada": {"inicio": 1, "fim": 3},
                "itens": [{"ocorrido_em": "…", "unidade": "…", "usuario": "… ou null", "descricao": "…"}]},
 "nao_lidos": ["marcadores", "propriedades", "documentos_dos_anexados"]}
```

`anexados` sai **sempre**, na ordem dos nós (`[]` sem anexado), também
com `--sem-documentos`: vem da mesma árvore, sem custo, e é identidade do
processo, não documento. Com algum anexado, `nao_lidos` termina em
`documentos_dos_anexados`; o texto humano lista cada um com a dica
`ies processo <NUP>`. `anexado` (`true` com o vínculo) e `anexado_a`
(`{"numero", "id_procedimento"}` do pai, ou `null`) também saem sempre;
o texto humano diz `Anexado ao processo <NUP> (id N): ies processo <NUP>`.

As formas de `andamentos` (objeto, `null` com motivo, ausente) estão em
[andamentos](processo-obter-andamentos.md).

`documentos` segue a ordem dos nós; com pastas, só sai depois de cada
pasta conferida pela identidade dos protocolos declarados. A dica dos documentos (a6) **não** é publicada:
em e-mails traz remetente e destinatários. `--sem-documentos` tira as
chaves `documentos`, `pastas` e `verificacao_das_pastas*` (a árvore continua lida: é ela que prova a identidade).
`--sem-marcadores` e `--sem-propriedades` são aceitos sem efeito.
`--sem-andamentos` com `--andamentos-completos` sai com 2
(`sem_andamentos_incompativel_com_andamentos_completos`).

## Falhas (saída 1)

Fluxo: `entrada_por_id_recusada_rode_ies_login`,
`sessao_encerrada_rode_ies_login`,
`resposta_da_entrada_por_id_nao_reconhecida`,
`pesquisa_rapida_sem_url_de_trabalho`, `redirecionamento_nao_reconhecido`,
`processo_repetido_no_controle`, `links_do_processo_repetidos_no_controle`,
`link_do_processo_no_controle_nao_reconhecido`,
`charset_do_controle_nao_suportado`. Os 302 conferidos usam os subcódigos
de [processo buscar](processo-buscar.md) e mais
`acao_de_origem_inesperada_no_redirecionamento` e
`id_procedimento_do_redirecionamento_diferente_do_pedido`; o formulário,
os do primitivo de formulário; Controle e transporte, os de sempre.

Tela: `arvore_ausente_na_tela_do_processo`,
`arvores_repetidas_na_tela_do_processo`,
`endereco_da_arvore_nao_reconhecido`,
`endereco_da_arvore_de_outro_processo`,
`html_da_tela_do_processo_nao_interpretado`,
`identificacao_da_unidade_ausente_na_tela_do_processo`,
`unidade_da_tela_diferente_da_solicitada`.

Árvore: `declaracao_de_no_fora_da_forma_observada`,
`argumento_de_no_nao_literal`, `texto_de_no_sem_fechamento`,
`escape_de_no_nao_reconhecido`,
`quantidade_de_argumentos_do_no_diferente_da_observada`,
`argumento_de_no_com_tipo_diferente_do_observado`,
`indice_de_no_fora_da_sequencia`, `tipo_de_no_da_arvore_nao_suportado`,
`no_do_processo_ausente_na_arvore`, `nos_do_processo_repetidos_na_arvore`,
`no_do_processo_com_pai`, `nup_do_no_do_processo_inconsistente`,
`id_de_no_nao_decimal`, `documento_com_pai_diferente_do_processo`,
`documento_repetido_na_arvore`,
`identidade_do_processo_diferente_da_pedida`,
`id_do_processo_na_arvore_diferente_do_aberto`.

Anexados: `processo_anexado_fora_da_forma_observada`,
`nup_do_no_do_processo_anexado_inconsistente`,
`processo_anexado_com_pai_diferente_do_processo`,
`processo_anexado_repetido_na_arvore`,
`processo_anexado_igual_ao_processo`, `no_com_pai_no_processo_anexado`,
`vinculo_de_anexacao_fora_da_forma_observada`,
`vinculos_de_anexacao_repetidos` (duas candidatas ou dois `.html` da raiz),
`html_do_no_do_processo_fora_da_forma_observada`.

Andamentos: os subcódigos da [seção própria](processo-obter-andamentos.md).
Pastas: os do [contrato das pastas](processo-obter-pastas.md).

Validação live da fatia 1: **PASS** na UnB, em GO e na PEN1 ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-arvore.md)).
Fatia 2 (andamentos): **PASS** ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-andamentos.md)).
Fatia 3 (pastas): **NOT_RUN** até a validação do agente pai.
Fatia 4 (colheita do Controle e passo 1b): **PASS** na UnB e na PEN1
([relatório](../../docs/validacao-live/2026-11-03-colheita-do-controle.md)).
Anexados: **PASS** na UnB nos dois sentidos ([relatório](../../docs/validacao-live/2026-11-03-processo-obter-anexados.md)).
Decisão em [processo.obter](../../docs/decisoes/processo-obter.md).

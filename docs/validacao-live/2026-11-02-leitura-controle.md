---
type: SEIObservation
title: Ensaio live da leitura do Controle em seis instalações de treinamento
description: PASS/FAIL/NOT_RUN de status, unidade atual e listar-processos com GET único, 02/11/2026
---

# Ensaio live da leitura do Controle — 02/11/2026

Primeiro ensaio **autenticado** do IES em instalações reais. Só
leitura: login + 3 GETs do IES por instalação, **zero POST, zero
escrita**. Binário `ies` em release a partir de `main` + #144.

## Condições

- **Login:** feito pelo Todos (`SEISession.caixa()`), usado como
  arnês privado fora do repositório; o IES não implementa login.
  Contas públicas de treinamento do
  [inventário do Todos](https://github.com/franklinbaldo/todos/blob/323eb013a1e7fac42c50336f838373a2cbe23e4b/docs/sei-multi-instalacao.md)
  (`aluno1`, `servidor1`, `orgao1`, `orgao3`, `orgao4`, `aluno05`).
- **Sessão para o IES:** origem, cookie e URL assinada do
  `procedimento_controlar` da **mesma geração**, em arquivos `0600`
  numa pasta `0700` fora do repositório (`IES_ORIGEM_FILE`,
  `IES_COOKIE_FILE`, `IES_URL_CONTROLE_FILE`). Nada disso, nem o HTML
  autenticado, foi versionado.
- **Comandos:** `ies status --verificar-controle-web --json`,
  `ies unidade atual --json`, `ies unidade listar-processos --json`.
- **Orçamento:** 3 GETs do IES ao Controle por instalação, além das
  requisições de login do Todos (≈3–5). Sem nova tentativa após falha,
  exceto uma única na PEN2 (503 no login) e, em GO, uma segunda rodada
  após a correção (login + 3 GETs + 2 GETs de diagnóstico só com
  status/tipo + 1 repetição manual).

## Resultado

| Instalação | SEI | Unidade exibida | `status --verificar-controle-web` | `unidade atual` | `unidade listar-processos` |
| --- | --- | --- | --- | --- | --- |
| UnB | 4.0.9 | Unidade1 | PASS | PASS | **PASS**: 12 recebidos, 47 gerados = legendas da tela |
| UFRJ | 4.1.5 | PR-P | PASS | PASS | **PASS**: 2 recebidos, 7 gerados = legendas |
| PEN1 | 4.0.12 | TESTE | PASS | PASS | **PASS (recusa)**: 114/218 registros, tela com 1–100 → `paginas_do_controle_nao_obtidas` |
| PEN3 | 5.0.4 | TESTE | PASS | PASS | **PASS (recusa)**: 171/123 registros, 1–100 → `paginas_do_controle_nao_obtidas` |
| PEN4 | 5.0.4 | TESTE | PASS | PASS | **PASS (recusa)**: 157/103 registros, 1–100 → `paginas_do_controle_nao_obtidas` |
| GO (1ª rodada) | 5.0.3 | — | **FAIL** `controle_web_nao_reconhecido` | **FAIL** `link_do_processo_nao_identificado_na_linha` | **FAIL** `resposta_inesperada` |
| GO (após a correção) | 5.0.3 | SEAD/CDE-09516 | PASS | PASS na 2ª execução; a 1ª deu `resposta_inesperada` | **PASS (recusa)**: 131/7990 registros, 1–10 → `paginas_do_controle_nao_obtidas` |
| PEN2 | 4.1.5 | — | NOT_RUN | NOT_RUN | NOT_RUN: HTTP 503 no login (2 tentativas) |

"PASS (recusa)" é o comportamento do contrato: com páginas não obtidas,
o IES falha fechado em vez de publicar lista parcial. Comprova, com
dados reais, a lacuna da paginação Web ativa (#57).

## Achados

**GO — link de processo sigiloso.** No Controle de Goiás, uma linha de
`tblProcessosGerados` tem o link `procedimento_trabalhar` com a classe
`processoVisualizadoSigiloso`; as outras 19 linhas usam
`processoVisualizado`. O leitor da caixa reconhecia só
`processoVisualizado`/`processoNaoVisualizado` e recusava a caixa
inteira. O protocolo de Goiás tem 15 dígitos sem pontuação; o IES já o
trata como opaco e não é a causa. A caixa de Goiás é paginada de 10 em
10 (131 recebidos, 7990 gerados).

Corrigido no mesmo dia: o leitor reconhece `processoVisualizadoSigiloso`
e expõe `sigiloso` no JSON v2. A nova rodada usou sessão nova e o
binário corrigido.

**GO — resposta intermitente.** Em 8 GETs do IES ao mesmo Controle
(duas sessões), 2 receberam `resposta_inesperada`. Dois GETs de
diagnóstico logo depois devolveram 200 com `text/html; charset=iso-8859-1`,
e a repetição do comando passou. O subcódigo atual não distingue status
HTTP, `Content-Type` ou decodificação; a causa fica indeterminada. Não
houve nova tentativa automática: a repetição foi manual e única.
O subcódigo foi então separado em status 5xx, 429, outro status, tipo
não HTML e corpo não decodificável. Numa 3ª rodada com esse binário
(login + 3 GETs), os três comandos passaram sem recorrência.

**Lista igual à tela.** Na UnB e na UFRJ, os totais devolvidos
coincidem com as legendas `N registros` das duas tabelas.

## Limites

- Uma conta e uma unidade por instalação; outras contas, perfis ou
  unidades podem divergir.
- O GET do Controle pode registrar efeitos administrativos; nenhum foi
  verificado ou observado neste ensaio.
- Não valida login do IES, paginação, pesquisa, POST nem escrita.
- O HTML de Goiás foi inspecionado só estruturalmente (nomes de
  atributos, classes e forma dos textos), sem copiar valores.

## Página observada e `--categoria T --pagina N` (#57)

Mesmo dia, mesmas sessões reaproveitadas (nenhum login novo), binário
com a página observada por padrão e `--pagina N` por categoria. Cada
`--pagina N` custou 1 GET + no máximo 1 POST; `N` fora do intervalo foi
recusado **antes** de qualquer POST. Nenhuma escrita.

> **Nota posterior (sintaxe):** este ensaio usou a sintaxe da PR #146,
> `ies unidade listar-processos --categoria T --pagina N`. Por decisão do
> proprietário, a mesma operação passou a `ies unidade recebidos|gerados
> --pagina N`, e `--categoria`/`--pagina` saíram de `listar-processos`
> ([listas](../../contracts/cli/unidade-recebidos-gerados.md)). O registro
> abaixo é histórico; a nova sintaxe ainda não foi ensaiada live.

| Instalação | Padrão (1 GET) | `--pagina N` | Fora do intervalo |
| --- | --- | --- | --- |
| UnB | 12/12 e 47/47, faixa 1–total | — | `gerados --pagina 2` → `pagina_fora_do_intervalo` |
| UFRJ | 2/2 e 7/7, faixa 1–total | — | idem |
| PEN1 | 100 de 114 e 100 de 218, faixa 1–100 | recebidos p2: 14 itens, 101–114; gerados p3: 18 itens, 201–218 | gerados p4 |
| PEN3 | 100 de 171 e 100 de 123 | recebidos p2: 71 itens, 101–171; gerados p2: 23 itens, 101–123 | gerados p3 |
| PEN4 | 100 de 157 e 100 de 103 | recebidos p2: 57 itens, 101–157; gerados p2: 3 itens, 101–103 | gerados p3 |
| GO | 10 de 131 e 10 de 7990, faixa 1–10 | gerados p799: 10 itens, 7981–7990; recebidos p14: 1 item, 131–131 | gerados p800 |

**Efeito de sessão observado (PEN1):** depois de `recebidos --pagina 2`,
a leitura padrão seguinte mostrou recebidos na faixa 101–114 — o
servidor lembra a página, e a saída o declara pela faixa. `--pagina 1`
em cada categoria restaurou 1–100 em todas as instalações paginadas.

## Nova sintaxe: `unidade recebidos|gerados [--pagina N]`

Mesmas sessões, sem login novo. Em PEN1, GO e UnB, `listar-processos`
devolveu o array das duas listas e `recebidos`/`gerados` uma lista cada,
com os mesmos totais e faixas das rodadas anteriores. PEN1:
`recebidos --pagina 2` 101–114, `gerados --pagina 3` 201–218,
`gerados --pagina 4` recusado sem POST, `--pagina 1` restaurou as duas.
UnB: `gerados --pagina 2` recusado sem POST. `listar-processos
--categoria` saiu com código 2.

**GO — 5xx ao voltar de 799 para 1.** `gerados --pagina 799` devolveu
7981–7990. Em seguida, `gerados --pagina 1` recebeu
`status_http_de_erro_do_servidor` duas vezes (a 2ª, repetição manual
única, após um GET que confirmou a página 799 lembrada). Na rodada
anterior, a mesma volta, depois de um POST em recebidos, passou. Sem
nova tentativa: a sessão de treinamento ficou com Gerados na página
799, e a leitura seguinte o declara pela faixa. O subcódigo separado
em #145 identificou a intermitência de GO como erro 5xx do servidor.

## Visão detalhada e mensagem do SEI nas recusas

**Visão detalhada (experimento, arnês privado, treinamento).** A troca
`hdnTipoVisualizacao=D` pelo formulário da caixa devolveu
`tblProcessosDetalhado` com total = recebidos + gerados (UnB 59, PEN1
332, GO 8121), **sem coluna de categoria** (na PEN1, ícones de status do
Tramita GOV.BR, não Recebidos/Gerados). Colunas variam: UnB 5, PEN1 4,
GO 10. Página: igual à resumida (PEN1 100, GO 10; UnB sem paginação).
**Persistência:** UnB 4.0.9 volta à resumida no GET seguinte; PEN1 5.0.4
e GO 5.0.3 mantêm a detalhada na sessão, e na PEN1 um **login novo** da
mesma conta abriu na detalhada. Todas as contas foram restauradas para a
resumida (POST `R`) e conferidas por GET; GO voltou à página 1 de Gerados.

**Mensagem do SEI nas recusas HTTP.** Com o binário que expõe
`status_http` e `mensagem_do_servidor`, a sequência que antes deu 5xx em
GO (`gerados --pagina 799` → `--pagina 1`) passou nas duas: validação
live da mensagem **NOT_RUN** (falha intermitente não recorreu).

**Categoria na visão D — não há.** Cruzando, na mesma sessão, cada linha
D com a categoria da resumida (UnB: 59/59 linhas, a caixa inteira; PEN1:
100 linhas), nenhum traço HTML (classes, atributos, ícones, `data-label`,
argumentos de tooltip, ocultos) separa Recebidos de Gerados. Os mais
correlatos são status alheios à categoria (ícone de trâmite recebido do
Tramita GOV.BR na PEN1; retorno programado na UnB). `hdnExibirRecebidos`/
`hdnExibirGerados` são os alternadores da resumida em tela pequena
(`alterarVisualizacaoTabela`, `d-md-none`): enviados com
`hdnTipoVisualizacao=D`, **não filtram** a D (UnB 59 e PEN1 332 em ambos).
A página da D fica lembrada à parte da resumida (PEN1 voltou em 101–200);
restaurada para a página 1 e a tela para R, conferidas por GET.

**Nível de detalhe.** A D publica `procedimento_configurar_detalhe`
(GET somente leitura): 12 opções fixas (`chkSinAtribuicao`, `chkSinAnotacao`,
`chkSinTipoProcesso`, `chkSinEspecificacao`, `chkSinNivelPrioritarios`,
`chkSinInteressados`, `chkSinObservacao`, `chkSinControlePrazo`,
`chkSinRetornoDevolver`, `chkSinRetornoAguardando`,
`chkSinUltimaMovimentacao`, `chkSinMarcadores`) e `sbmSalvar`. PEN1 tinha 3
marcadas, GO todas; GO com 12 marcadas mostra 9 colunas além de Processo
(prazo, devolver e aguardando não viram cabeçalho). Nada foi salvo.

## Roteiro: visão detalhada por padrão

Para o operador, em treinamento, depois da PR da visão detalhada
([contrato](../../contracts/cli/unidade-listar-processos-visao-detalhada.md)).
Mesmas condições acima: arquivos `0600` em pasta `0700`, sessão da mesma
geração, binário release. O script não põe cookie nem URL assinada em
argumentos de processo (vão num arquivo de configuração do `curl` `0600`), e
só imprime visão, índices de página, contagens e subcódigos. Cada `estado`
é **1 GET** a mais, de conferência.

```sh
#!/bin/sh
# uso: IES_ORIGEM_FILE=... IES_COOKIE_FILE=... IES_URL_CONTROLE_FILE=... sh roteiro.sh
set -u
umask 077
T=$(mktemp -d) && trap 'rm -rf "$T"' EXIT
printf 'url = "%s"\nheader = "Cookie: %s"\nproto = "=https"\nmax-redirs = 0\nsilent\n' \
  "$(cat "$IES_URL_CONTROLE_FILE")" "$(cat "$IES_COOKIE_FILE")" > "$T/curl.cfg"
estado() {  # 1 GET: só hdnTipoVisualizacao e os índices de página
  curl -K "$T/curl.cfg" -o "$T/tela.html" || { echo "estado: GET falhou"; return; }
  python3 -I -c '
import sys
t = open(sys.argv[1], encoding="latin-1").read()
for c in ("hdnTipoVisualizacao", "hdnRecebidosPaginaAtual",
          "hdnGeradosPaginaAtual", "hdnDetalhadoPaginaAtual"):
    i = t.find("name=\"%s\" value=\"" % c)
    print(c, t[i + len(c) + 15:].split("\"", 1)[0] if i >= 0 else "-", end="; ")
print()' "$T/tela.html"
}
resumo() {  # contagens do JSON, sem valores de processos
  python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1]))
if isinstance(v, dict) and "codigo" in v:
    print("FALHA", v.get("subcodigo"), v.get("causa"), v.get("visao_deixada"), v.get("status_http"))
elif isinstance(v, dict) and "opcoes" in v:
    print("colunas marcadas:", [o["coluna"] for o in v["opcoes"] if o["marcada"]])
else:
    l = v[0] if isinstance(v, list) else v
    ps = l["processos"]
    cats = {}
    for p in ps:
        cats[p.get("categoria")] = cats.get(p.get("categoria"), 0) + 1
    chaves = sorted({k for p in ps for k in p.get("colunas", {})})
    print("total", l["total_declarado"], "faixa", l["faixa_observada"], "linhas", len(ps),
          "categorias", cats, "colunas", chaves, "ausentes", l.get("colunas_ausentes"))' "$1"
}
passo() {  # passo NOME ARGS...: roda o ies, resume e confere o estado
  nome=$1; shift
  ies --json "$@" > "$T/$nome.json"; echo "$nome: saída $?"; resumo "$T/$nome.json"; estado
}
estado                                   # E0: visão e páginas encontradas
passo lp unidade listar-processos        # R: 1 GET + 2 POST; D: 1 GET
passo lp2 unidade listar-processos --pagina 2   # R: 1 GET + 4 POST após a correção (3 no ensaio)
passo fora unidade listar-processos --pagina 99999  # pagina_fora_do_intervalo, sem POST
passo col unidade colunas                # R: 2 GET + 2 POST; D: 2 GET
passo rec unidade recebidos              # R: 1 GET; D: 1 GET + 2 POST
passo ger2 unidade gerados --pagina 2    # muda a página lembrada de Gerados (efeito conhecido)
passo ger1 unidade gerados --pagina 1    # volta Gerados à página 1
```

Critério: depois de cada `passo`, `estado` igual ao E0 (mesma visão; mesmas
páginas de Recebidos/Gerados na R; mesma `hdnDetalhadoPaginaAtual` na D),
exceto `ger2`, que muda só `hdnGeradosPaginaAtual` (desfeito por `ger1`).
`lp`: categorias por processo somam o que a R mostrou; `colunas` iguais às
marcadas em `col` (as de ícone podem faltar na PEN1, divergência a
registrar). Repetir com a sessão **já na D** (PEN1 ou GO: ligar a
Visualização detalhada pela interface ou pelo arnês), conferindo que
`lp` não faz POST, que `lp2` volta a D à página exibida e que `rec`/`ger2`
deixam a D como estava; no fim, voltar a sessão à R pela interface e
conferir com `estado`. Qualquer `visao_do_controle_nao_restaurada` ou
`pagina_da_visao_detalhada_nao_restaurada`: parar, registrar `causa` e
`status_http`, restaurar manualmente e conferir com `estado`. Sem retry.

## Ensaio da visão detalhada por padrão

Roteiro acima executado pelo operador em 02/11/2026, nas instalações de
treinamento, com o binário da PR da visão detalhada. Sem valores reais.

**PEN1 (5.0.4), sessão na R.** `listar-processos`: 1 GET + 2 POST, 100 de
332 processos, faixa 1–100; categoria por processo pela R do mesmo GET: 77
recebidos e 23 gerados; colunas `atribuicao`, `interessados` e `tipo`;
`colunas_ausentes` com 6 chaves. `--pagina 2`: faixa 101–200, 14 processos
sem categoria (não estavam na R do GET). `--pagina 99999`:
`pagina_fora_do_intervalo`. `colunas`: 3 marcadas. `recebidos` e `gerados`
sem mudança. Estado R / página 0 / página 0 depois de cada comando, **exceto**
a página lembrada da D: o `--pagina 2` (GET, POST D, POST D página 2, POST
R) deixou `hdnDetalhadoPaginaAtual=1`, e o `listar-processos` seguinte (GET,
POST D, POST R) devolveu a faixa 101–200 em vez da 1–100 que a D mostrava
antes; a D do navegador também abriria na página 2. **FAIL** de "deixar
como encontrou", corrigido na revisão seguinte: volta à página encontrada
da D antes do POST R, e o mesmo para a página da R em `recebidos`/`gerados
--pagina N` com a sessão na D ([contrato](../../contracts/cli/unidade-listar-processos-visao-detalhada.md)).

**UnB (4.0.9).** `listar-processos`: 59 de 59, todos com categoria (12
recebidos, 47 gerados); colunas de ícone lidas (`controle_de_prazo`,
`para_devolver`, `aguardando_retorno`). `--pagina 2`:
`pagina_fora_do_intervalo`. `colunas`: **FAIL** com
`opcao_do_nivel_de_detalhe_ausente`: a tela publica 11 caixas, sem
`chkSinNivelPrioritarios` (Prioridade); PEN1 e GO publicam as 12. Corrigido
na revisão seguinte: o catálogo de 12 é o conhecido, a tela pode publicar um
subconjunto, e cada opção sai com `disponivel`
([contrato](../../contracts/cli/unidade-colunas.md)).

**GO (5.0.3).** `listar-processos`: 10 de 8121, as 12 colunas,
`colunas_ausentes: []`. `--pagina 2`: faixa 11–20. Estado restaurado. Um
503 intermitente no GET de conferência do operador (fora do IES).

Reensaio das duas correções: **NOT_RUN**.

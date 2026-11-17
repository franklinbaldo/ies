---
type: SEIObservation
title: Leitura do Controle — visão detalhada por padrão
description: Roteiro e ensaio da visão detalhada por padrão, definir colunas e sem restauração na PEN1
---

# Leitura do Controle — visão detalhada por padrão

Parte do [ensaio live da leitura do Controle](../2026-11-02-leitura-controle.md), separada por responsabilidade.

## Roteiro: visão detalhada por padrão

Para o operador, em treinamento, depois da PR da visão detalhada
([contrato](../../../contracts/cli/unidade-listar-processos-visao-detalhada.md)).
Mesmas condições acima: arquivos `0600` em pasta `0700`, sessão da mesma
geração, binário release. O script não põe cookie nem URL assinada em
argumentos de processo (vão num arquivo de configuração do `curl` `0600`), e
só imprime visão, índices de página, contagens e subcódigos. Cada `estado`
é **1 GET** a mais, de conferência. As rodadas de 02/11/2026 abaixo usaram a
versão que **restaurava** visão e páginas; desde a decisão do proprietário
de não restaurar ([decisão](../../decisoes/unidade-listar-processos.md)), os
orçamentos e o critério são os comentados no script.

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
passo lp unidade listar-processos        # R: 1 GET + 1 POST, fica na D; D: 1 GET
passo lp2 unidade listar-processos --pagina 2   # D: 1 GET + 1 POST, D fica na página 2
passo lp1 unidade listar-processos --pagina 1   # D: 1 GET + 1 POST, volta à página 1
passo fora unidade listar-processos --pagina 99999  # pagina_fora_do_intervalo, sem POST
passo col unidade colunas                # D: 2 GET (R: 2 GET + 1 POST, fica na D)
passo rec unidade recebidos              # D: 1 GET + 1 POST, fica na R; R: 1 GET
passo ger2 unidade gerados --pagina 2    # R: 1 GET + 1 POST, Gerados fica na página 2
passo ger1 unidade gerados --pagina 1    # volta Gerados à página 1
```

Critério (sem restauração): partindo da R, `lp` deixa a sessão na D (na UnB
4.0.9 o GET seguinte já volta à R); `lp2` deixa `hdnDetalhadoPaginaAtual=1`
e faixa 101–200 (PEN1), desfeitos por `lp1`; `col` não troca de visão
(sessão já na D, 2 GET); `rec` deixa a sessão na R; `ger2` muda só
`hdnGeradosPaginaAtual` (desfeito por `ger1`). Cada `estado` mostra a visão
e as páginas que o último comando deixou, e a `faixa_observada` de cada
saída diz qual página foi lida.
`lp`: categorias por processo somam o que a R mostrou; `colunas` iguais às
marcadas em `col` (as de ícone podem faltar na PEN1, divergência a
registrar). Repetir com a sessão **já na D** (PEN1 ou GO: ligar a
Visualização detalhada pela interface ou pelo arnês), conferindo que
`lp` não faz POST e que `lp2` faz um só; no fim, voltar a sessão à R pela
interface, se desejado, e conferir com `estado`. Falha: parar, registrar
subcódigo e `status_http`, conferir com `estado`. Sem retry.

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
--pagina N` com a sessão na D ([contrato](../../../contracts/cli/unidade-listar-processos-visao-detalhada.md)).

**UnB (4.0.9).** `listar-processos`: 59 de 59, todos com categoria (12
recebidos, 47 gerados); colunas de ícone lidas (`controle_de_prazo`,
`para_devolver`, `aguardando_retorno`). `--pagina 2`:
`pagina_fora_do_intervalo`. `colunas`: **FAIL** com
`opcao_do_nivel_de_detalhe_ausente`: a tela publica 11 caixas, sem
`chkSinNivelPrioritarios` (Prioridade); PEN1 e GO publicam as 12. Corrigido
na revisão seguinte: o catálogo de 12 é o conhecido, a tela pode publicar um
subconjunto, e cada opção sai com `disponivel`
([contrato](../../../contracts/cli/unidade-colunas.md)).

**GO (5.0.3).** `listar-processos`: 10 de 8121, as 12 colunas,
`colunas_ausentes: []`. `--pagina 2`: faixa 11–20. Estado restaurado. Um
503 intermitente no GET de conferência do operador (fora do IES).

Reensaio das duas correções: **NOT_RUN**.

**Reensaio com as correções (mesmas sessões).** PEN1: `listar-processos
--pagina 2` (101–200) seguido do padrão devolveu 1–100, tela R/0/0 — a
página da D voltou à encontrada. UnB: `unidade colunas` com 11 opções
(sem Prioridade) passou. GO: a mensagem dos 5xx intermitentes ficou
visível — `HTTP 503: upstream connect error or disconnect/reset before
headers ... connection timeout`, texto de proxy reverso, não do SEI. Um
503 caiu no POST de volta à página encontrada: o IES devolveu
`pagina_da_visao_detalhada_nao_restaurada` com `visao_deixada: "R"`,
causa, status e mensagem; a tela ficou R/0/0 (conferida por GET), e a
página lembrada da D de GO ficou em 2; limpa às 18:53 pelo arnês (D → página 1 → R), conferida por GET.
Sem novas tentativas.

## `unidade colunas --definir` (escrita do Nível de detalhe)

Sessões reaproveitadas; contas de treinamento. Para cada conta: leitura
da configuração encontrada, prévia, `--definir todas --confirmar`,
leitura, `listar-processos`, restauração com `--definir <original>
--confirmar`, leitura e GET de estado.

| Instalação | Encontrada | Prévia | Aplicar `todas` | `colunas_ausentes` | Restaurar | Estado final |
| --- | --- | --- | --- | --- | --- | --- |
| PEN1 5.0.4 | 3 marcadas | 9 a marcar, 14 campos, nada gravado | exit 0, 12 marcadas, conferido por leitura | `[]` | exit 0, as 3 originais | R/0/0 |
| UnB 4.0.9 | 7 marcadas | 4 a marcar, 13 campos, nada gravado | exit 0, 11 disponíveis marcadas | `["prioridade"]` (inexistente no 4.0.9) | exit 0, as 7 originais | R/0/0 |

Nenhuma reconciliação foi necessária (`reconciliado_por_leitura: false`).
GO não foi ensaiado (instável na rodada anterior).

## Sem restauração, com sessão do `ies login` (PEN1)

Só IES, sem o arnês do Todos: `ies login` (6 requisições) e, em
sequência, `listar-processos` (R→D, fica em D; 77 recebidos/23 gerados
por processo, vindos da R do mesmo GET), `listar-processos` (1 GET,
`categoria: null` em todos, sessão já em D), `recebidos` (D→R, fica em
R), `listar-processos --pagina 2` (101–200, D na página 2) e
`listar-processos` (101–200: página lembrada, declarada pela faixa).
Estado conferido por GET após cada comando, igual ao previsto.

---
type: Guide
title: Roteiro live de ies unidade colunas --definir
description: Registrar a configuração encontrada, aplicar todas com --confirmar, verificar e restaurar a original, conferindo a tela do Controle
---

# Roteiro live: `ies unidade colunas --definir` (Refs #57)

Para o operador, **só em treinamento** (PEN1 5.0.4 primeiro; depois GO e a
UnB 4.0.9, que publica 11 opções), depois da PR da escrita
([contrato](../../contracts/cli/unidade-definir-colunas.md),
[decisão](../decisoes/unidade-definir-colunas.md)). Mesmas condições do
[ensaio da leitura](2026-11-02-leitura-controle.md): arquivos `0600` em pasta
`0700`, sessão da mesma geração, binário release. O script não põe cookie nem
URL assinada em argumentos de processo e só imprime visão, índices de página,
chaves de colunas, contagens e subcódigos. **Duas escritas** de preferência:
aplicar `todas` e restaurar a original.

```sh
#!/bin/sh
# uso: IES_ORIGEM_FILE=... IES_COOKIE_FILE=... IES_URL_CONTROLE_FILE=... sh roteiro-colunas.sh
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
lista() {  # lista CAMPO ARQUIVO: chaves com o campo verdadeiro em `opcoes`
  python3 -I -c '
import json, sys
v = json.load(open(sys.argv[2]))
print(",".join(o["coluna"] for o in v.get("opcoes", []) if o[sys.argv[1]]))' "$1" "$2"
}
resumo() {  # saída de --definir, sem valores de processos
  python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1]))
if "codigo" in v:
    print("FALHA", v.get("codigo"), v.get("subcodigo"), "status", v.get("status_http"),
          "salvar", (v.get("falha_do_salvar") or {}).get("subcodigo"),
          "verificacao", v.get("verificacao"), "observadas", v.get("observadas"),
          "restauracao", v.get("restauracao"), "visao_deixada", v.get("visao_deixada"))
else:
    print("confirmado", v["confirmado"], "alterado", v["alterado"],
          "reconciliado", v["reconciliado_por_leitura"], "marcar", v["marcar"],
          "desmarcar", v["desmarcar"], "indisponiveis", v["indisponiveis"],
          "campos", (v.get("envio") or {}).get("campos"), "depois", v.get("depois"))' "$1"
}
colunas() {  # colunas NOME: ies unidade colunas (2 GET; vinda da R, + 2 POST)
  ies --json unidade colunas > "$T/$1.json"; echo "$1: saída $?"
}
confere() {  # confere ROTULO: tela do Controle igual à encontrada
  estado > "$T/e"; if cmp -s "$T/e0" "$T/e"; then echo "$1: tela como encontrada"
  else echo "$1: TELA DIFERENTE"; cat "$T/e"; fi
}

estado > "$T/e0"; cat "$T/e0"                       # E0
colunas c0
ORIGINAL=$(lista marcada "$T/c0.json"); TODAS=$(lista disponivel "$T/c0.json")
echo "original: $ORIGINAL"; echo "disponíveis: $TODAS"
[ -n "$ORIGINAL" ] || { echo "PARAR: nenhuma marcada; --definir não aceita lista vazia"; exit 1; }
confere c0

# 1. Prévia: zero POST de salvar (D: 2 GET; R: 2 GET + 2 POST).
ies --json unidade colunas --definir todas > "$T/p.json"; echo "previa: saída $?"
resumo "$T/p.json"; confere previa
colunas c1; [ "$(lista marcada "$T/c1.json")" = "$ORIGINAL" ] && echo "prévia: configuração intacta" \
  || echo "PRÉVIA ALTEROU A CONFIGURAÇÃO"

# 2. Aplicar todas (D: 3 GET + 1 POST; R: 3 GET + 3 POST).
ies --json unidade colunas --definir todas --confirmar > "$T/a.json"; rc=$?
echo "aplicar: saída $rc"; resumo "$T/a.json"; confere aplicar
colunas c2; AGORA=$(lista marcada "$T/c2.json"); echo "lido depois: $AGORA"
if [ "$rc" = 3 ]; then
  echo "EFEITO INCERTO: não repetir. Registrar e restaurar à mão com:"
  echo "  ies unidade colunas --definir $ORIGINAL --confirmar"; exit 3
fi
[ "$AGORA" = "$TODAS" ] && echo "verificado: todas as disponíveis marcadas" || echo "NÃO SÃO TODAS"
ies --json unidade listar-processos > "$T/lp.json"; echo "listar: saída $?"
python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1])); l = v[0] if isinstance(v, list) else v
print("colunas_ausentes", l.get("colunas_ausentes"),
      "colunas", sorted({k for p in l.get("processos", []) for k in p.get("colunas", {})}))' "$T/lp.json"
confere listar

# 3. Restaurar a original, mesmo se o passo 2 falhou com 1 (o estado é conhecido).
if [ "$AGORA" != "$ORIGINAL" ]; then
  ies --json unidade colunas --definir "$ORIGINAL" --confirmar > "$T/r.json"; rc=$?
  echo "restaurar: saída $rc"; resumo "$T/r.json"
  [ "$rc" = 3 ] && { echo "EFEITO INCERTO na restauração: não repetir; conferir e restaurar à mão"; exit 3; }
fi
colunas c3; [ "$(lista marcada "$T/c3.json")" = "$ORIGINAL" ] && echo "configuração restaurada" \
  || echo "CONFIGURAÇÃO NÃO RESTAURADA: $(lista marcada "$T/c3.json")"
confere final
```

Critério: `previa` com `confirmado false`, `campos` iguais a
`sbmSalvar`, as caixas de `TODAS` e `hdnInfraTipoPagina` (ordem do
documento) e configuração intacta; `aplicar` com saída 0, `alterado true`,
`depois` = `TODAS` e `reconciliado false`; `c2` = `TODAS`;
`colunas_ausentes` vazia em `listar` (as de ícone podem faltar, divergência
já registrada); `restaurar` com saída 0 e `c3` = `ORIGINAL`; todo `confere`
igual ao E0. Repetir com a sessão **já na D** (PEN1 ou GO), conferindo que
nenhum `POST visao` acontece. Registrar, além do PASS/FAIL, se a janela
publica `lnkInfraUnidade`, a estrutura (não o conteúdo) da resposta do
salvar e qualquer subcódigo. Saída 3: parar, sem retry; registrar `status`,
`salvar` e `verificacao`, e restaurar à mão conferindo com `colunas`.

Orçamento total por instalação (sessão em R): 6 GETs de `estado`, 4
`colunas` (2 GET + 2 POST cada), prévia (2 GET + 2 POST), duas escritas
(3 GET + 3 POST cada) e `listar-processos` (1 GET + 2 POST). Na D, sem os
POSTs de troca. Resultado: **NOT_RUN** até o ensaio.

---
type: Guide
title: Roteiro live de ies unidade colunas --definir
description: Aplicar todas as colunas com --confirmar uma vez por instalação, guardar a saída inteira, verificar por leitura e não restaurar
---

# Roteiro live: `ies unidade colunas --definir` (Refs #57)

Para o operador, **só em treinamento** e com autorização para **uma
escrita** de preferência por instalação
([contrato](../../contracts/cli/unidade-definir-colunas.md),
[decisão](../decisoes/unidade-definir-colunas.md)). Ordem: PEN3 e PEN4
(5.0.4, sessões novas), depois GO e a UnB 4.0.9 (11 opções). A PEN1 só
volta depois de um PASS nas outras
([análise da saída 3 de 04/11](2026-11-04-definir-colunas-pen1.md)).
PEN4: PASS em 04/11/2026, 12 colunas com 1 POST e sem reconciliação
([relatório](2026-11-04-definir-colunas-pen4.md)).

**Não há restauração.** Todas as colunas são o estado final desejado
(decisão do proprietário "B"; o IES não restaura tela, visão nem colunas).
Se a instalação já tiver todas marcadas, o `--confirmar` não envia POST
(`alterado: false`, `envio: null`): o salvar fica **NOT_RUN** ali, e tirar
uma coluna só para reensaiar é outra escrita, que precisa de autorização
própria.

Condições: arquivos `0600` em pasta `0700`, sessão da mesma geração,
binário release do ramo, **22 s ou mais entre comandos** (uma rajada pode
cair num bloqueio de borda e transformar o salvar em efeito incerto). A
saída JSON inteira de cada comando vai para um arquivo privado: ela não traz
cookie, URL nem `infra_hash`, e é o que permite diagnosticar uma saída 1 ou
3. O script imprime só chaves de colunas, contagens, subcódigos e status.

```sh
#!/bin/sh
# uso: D=<pasta da sessão> sh roteiro-colunas.sh   (origem, cookie, url-caixa, cred)
set -u
umask 077
export IES_ORIGEM_FILE=$D/origem IES_COOKIE_FILE=$D/cookie \
  IES_URL_CONTROLE_FILE=$D/url-caixa IES_CREDENCIAIS_FILE=$D/cred IES_SEM_INDICE=1
O=$D/definir-$(date +%Y%m%dT%H%M%S); mkdir -m 700 "$O"
pausa() { sleep 22; }
lista() {  # lista CAMPO ARQUIVO: chaves com o campo verdadeiro em `opcoes`
  python3 -I -c '
import json, sys
v = json.load(open(sys.argv[2]))
print(",".join(o["coluna"] for o in v.get("opcoes", []) if o[sys.argv[1]]))' "$1" "$2"
}
resumo() {  # resumo ARQUIVO: sucesso ou falha, com as causas
  python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1]))
if "codigo" in v:
    print("FALHA", v.get("codigo"), v.get("subcodigo"), "status", v.get("status_http"),
          "salvar", v.get("falha_do_salvar"), "verificacao", v.get("verificacao"),
          "observadas", v.get("observadas"))
else:
    print("confirmado", v["confirmado"], "alterado", v["alterado"],
          "reconciliado", v["reconciliado_por_leitura"], "salvar", v.get("falha_do_salvar"),
          "marcar", v["marcar"], "indisponiveis", v["indisponiveis"],
          "campos", (v.get("envio") or {}).get("campos"), "depois", v.get("depois"))' "$1"
}

ies --json login > "$O/login.json"; echo "login: saída $?"; pausa
ies --json unidade colunas > "$O/c0.json"; echo "c0: saída $?"; pausa
ANTES=$(lista marcada "$O/c0.json"); TODAS=$(lista disponivel "$O/c0.json")
echo "antes: $ANTES"; echo "disponíveis: $TODAS"
[ "$ANTES" = "$TODAS" ] && { echo "NOT_RUN: todas já marcadas; nenhum POST a ensaiar"; exit 0; }

ies --json unidade colunas --definir todas > "$O/previa.json"; echo "prévia: saída $?"
resumo "$O/previa.json"; pausa

ies --json unidade colunas --definir todas --confirmar > "$O/aplicar.json"; rc=$?
echo "aplicar: saída $rc"; resumo "$O/aplicar.json"; pausa
ies --json unidade colunas > "$O/c1.json"; echo "c1: saída $?"
DEPOIS=$(lista marcada "$O/c1.json"); echo "lido depois: $DEPOIS"
if [ "$rc" != 0 ]; then
  echo "PARAR: saída $rc; não repetir. Guardar $O e registrar os subcódigos acima."; exit "$rc"
fi
[ "$DEPOIS" = "$TODAS" ] && echo "verificado: todas as disponíveis marcadas" || echo "NÃO SÃO TODAS"
pausa
ies --json unidade listar-processos > "$O/lp.json"; echo "listar: saída $?"
python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1])); l = v[0] if isinstance(v, list) else v
print("colunas_ausentes", l.get("colunas_ausentes"))' "$O/lp.json"
```

## Critério

- **prévia:** saída 0, `confirmado false`, `campos` = `hdnInfraTipoPagina`,
  `sbmSalvar` e as caixas de `TODAS`, na ordem do documento; nada gravado.
- **aplicar (PASS):** saída 0, `confirmado true`, `alterado true`,
  `reconciliado false`, sem `falha_do_salvar`, `depois` = `TODAS`; `c1` =
  `TODAS`; `colunas_ausentes` vazia em `listar` (as de ícone podem faltar,
  divergência já registrada).
- **PASS reconciliado:** saída 0 com `reconciliado true`: gravou, mas a
  resposta não trouxe o sinal (`falha_do_salvar` diz qual). Registrar o
  subcódigo; em `resposta_do_salvar_nao_reconhecida`, a versão responde de
  outro jeito e o contrato precisa da estrutura (não do conteúdo) dela.
- **FAIL:** saída 1 (`configuracao_de_colunas_nao_aplicada` ou
  `escrita_diferente_da_solicitada`), com `falha_do_salvar`.
- **Saída 3:** parar, sem retry; registrar `falha_do_salvar` e
  `verificacao` do arquivo `aplicar.json`; o `c1` (22 s depois) diz o
  estado. A sessão continua: nada a desfazer.

## Orçamento por instalação

Sessão na D: `login` (1 a 7 requisições), `c0` 2 GET, prévia 2 GET,
aplicar 3 GET + **1 POST de salvar**, `c1` 2 GET, `listar-processos` 1 GET:
cerca de 10 GET + 1 POST além do login, seis comandos com 22 s entre eles.
Vinda da R, mais **1 POST D** no `c0` (a sessão fica na D). Nenhum comando
é repetido. Resultado por instalação: PASS, FAIL ou NOT_RUN no relatório
live do dia.

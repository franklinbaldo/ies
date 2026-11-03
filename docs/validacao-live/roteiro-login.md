---
type: Guide
title: Roteiro live de ies login
description: Login, unidade atual, conferência sem novo login, login forçado e lock, por instalação de treinamento, sem expor credencial nem sessão
---

# Roteiro live: `ies login`

Para o operador, **só em treinamento**, uma instalação por vez, depois da
PR do login ([contrato](../../contracts/cli/sessao-login.md),
[decisão](../decisoes/sessao-login.md)). As contas são as **públicas** de
treinamento publicadas pelas instalações (UnB:
<https://portalsei.unb.br/ambiente-de-treinamento-do-sei-v-4-0/>; PEN:
wiki do Tramita.GOV.BR; UFRJ:
<https://portal.sei.ufrj.br/capacitacao-e-orientacao/ambiente-de-testes/>;
GO: <https://sei.goias.gov.br/material_para_treinamento.php>), as mesmas do
`scripts/capturar_instalacoes.py` do Todos. Uma tentativa por conta:
`credencial_recusada`, `captcha_exigido` ou `segundo_fator_exigido` param
o roteiro daquela instalação, sem repetir.

O script grava origem e credenciais em arquivos `0600` de uma pasta
temporária `0700` (o `printf` é embutido do shell: a senha não vira
argumento de processo), não imprime cookie, URL nem senha, e apaga a pasta
ao sair. Só mostra `conexao`, `requisicoes`, `unidade_exibida`,
subcódigos, `status_http` e a mensagem saneada.

```sh
#!/bin/sh
# uso: sh roteiro-login.sh unb|pen1|ufrj|go   (binário release no PATH)
set -u
umask 077
case "${1:-}" in
  unb)  H=sei.treinamento.unb.br; U=aluno1; S=aluno1; O=UnB ;;
  pen1) H=sei.orgao1.tramita.processoeletronico.gov.br; U=orgao1; S=orgao1; O=ORGAO1 ;;
  ufrj) H=teste.sei.tic.ufrj.br; U=servidor1; S=servidor1; O=UFRJ ;;
  go)   H=seitreinamento.go.gov.br; U=aluno05; S=abc123456; O=SEAD ;;
  *) echo "uso: $0 unb|pen1|ufrj|go"; exit 2 ;;
esac
D=$(mktemp -d) && trap 'rm -rf "$D"' EXIT
printf 'https://%s/sei\n' "$H" > "$D/origem"
printf 'usuario=%s\nsenha=%s\norgao=%s\n' "$U" "$S" "$O" > "$D/credenciais"
unset S IES_CONFIG_FILE
export IES_ORIGEM_FILE="$D/origem" IES_COOKIE_FILE="$D/cookie" \
  IES_URL_CONTROLE_FILE="$D/url-controle" IES_CREDENCIAIS_FILE="$D/credenciais"
passo() {  # passo ROTULO comando...: saída e só os campos públicos
  r=$1; shift
  "$@" > "$D/$r.json"; rc=$?
  printf '%s: saída %s, ' "$r" "$rc"
  python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1]))
print({k: v[k] for k in ("conexao", "requisicoes", "unidade_exibida", "instalacao",
       "subcodigo", "status_http", "mensagem_do_servidor", "campo") if k in v})' "$D/$r.json"
  return "$rc"
}
passo 1-login ies --json login || exit 1          # login_realizado
python3 -I -c '
import json, os, sys
for n in ("cookie", "url-controle", "url-controle.geracao"):
    print(n, oct(os.stat(os.path.join(sys.argv[1], n)).st_mode & 0o777))
print("geracao", json.load(open(os.path.join(sys.argv[1], "url-controle.geracao")))["geracao"])' "$D"
passo 2-unidade ies --json unidade atual          # a mesma unidade
passo 3-de-novo ies --json login                  # sessao_existente_valida, requisicoes 1
passo 4-forcar ies --json login --forcar          # login_realizado, geracao 2
python3 -I -c 'import json,sys; print("geracao", json.load(open(sys.argv[1]))["geracao"])' \
  "$D/url-controle.geracao"
passo 5-unidade ies --json unidade atual          # a sessão nova vale
# Lock: dois processos ao mesmo tempo; os dois conferem a sessão (1 GET cada).
ies --json login > "$D/6a.json" & ies --json login > "$D/6b.json"; wait
for r in 6a 6b; do python3 -I -c '
import json, sys
v = json.load(open(sys.argv[1])); print(sys.argv[2], v.get("conexao"), v.get("requisicoes"), v.get("subcodigo"))' \
  "$D/$r.json" "$r"; done
```

## Critério e registro

PASS por instalação: passo 1 `login_realizado` com `requisicoes` no
orçamento da família ([contrato](../../contracts/cli/sessao-login.md):
UnB 2 + 1 + saltos depois do POST; PEN1, UFRJ e GO 3 + 1 + saltos; +1 se a
conta abrir no Painel), os três arquivos `600`, `geracao 1`; passo 2 com a
mesma `unidade_exibida`; passo 3 `sessao_existente_valida` com
`requisicoes 1`; passo 4 `login_realizado` e `geracao 2`; passo 5 igual ao
2; passos 6a e 6b `sessao_existente_valida`. Registrar, por instalação,
versão observada, `requisicoes` de cada passo, tempo, se a conta abriu no
Painel e qualquer subcódigo. Falha: registrar o subcódigo, `status_http` e
a mensagem, sem repetir o passo. Sem acesso ou autorização: `NOT_RUN`.

## Resultado live (02/11/2026, treinamento)

Roteiro executado uma vez por instalação, com o binário release.

| Instalação | Família | 1 login | 3 sessão existente | 4 `--forcar` | 6 lock (2 processos) | Unidade |
| --- | --- | --- | --- | --- | --- | --- |
| UnB 4.0.9 | SIP pela raiz | `login_realizado`, 6 requisições, arquivos 0600, geração 1 | `sessao_existente_valida`, 1 | `login_realizado`, 6, geração 2 | ambos `sessao_existente_valida`, 1 cada | Unidade1 |
| PEN1 5.0.4 | SIP via `/sei/` | idem, 6 | idem, 1 | idem, 6 | idem | TESTE |
| UFRJ 4.1.5 | SIP via `http://` promovido | idem, 6 | idem, 1 | idem, 6 | idem | PR-P |
| GO 5.0.3 | módulo próprio | idem, 6 | idem, 1 | idem, 6 | idem | SEAD/CDE-09516 |

`unidade atual` após cada login confirmou a sessão. O SIP aceitou o POST
sem `sbmAcessar` em PEN1, UFRJ e GO (como o navegador).

Senha errada (uma tentativa, UnB): `credencial_recusada`, saída 1,
nenhum arquivo de sessão gravado; `mensagem_do_servidor` ausente: o SIP
devolveu a mensagem no textarea oculto `txaInfraValidacao`, que o IES
não lia.

## Lacunas fechadas offline (revalidação live pendente)

- **Mensagem da credencial recusada**: `credencial_recusada` agora lê o
  [`txaInfraValidacao`](../como-o-sei-funciona/validacao-da-infra.md),
  com fixture sintética na estrutura observada. Revalidar com **uma**
  tentativa de senha errada na UnB (esperado: `mensagem_do_servidor`
  com o texto genérico de usuário ou senha inválida) e, se autorizado,
  numa instalação de outra família. `NOT_RUN` até lá.
- **Proxy**: em rede com saída só por proxy, o login conectava direto.
  Agora segue `https_proxy`/`HTTPS_PROXY`/`all_proxy`/`ALL_PROXY` e
  `no_proxy`/`NO_PROXY` ([contrato](../../contracts/cli/sessao-login.md#proxy-do-ambiente)).
  Revalidar o passo 1 com o proxy exportado: esperado `login_realizado`;
  um `proxy_*` registra o subcódigo e `status_do_proxy`. Os outros
  comandos (`unidade atual`) **não** usam proxy: numa rede sem saída
  direta, o passo 2 falhará com `falha_de_rede`. `NOT_RUN`.

**Reensaio das lacunas (02/11/2026, UnB).** Com o binário que lê
`txaInfraValidacao` e atravessa proxy: senha errada (1 tentativa) →
`credencial_recusada` com `mensagem_do_servidor: "Usuário ou Senha
Inválida."`; login correto **pelo proxy HTTPS do ambiente** (túnel
CONNECT) → `login_realizado`, 6 requisições; o mesmo com `NO_PROXY='*'`
→ conexão direta, `login_realizado`, 6. Os demais comandos (minreq sem a
feature `proxy`) continuam conectando direto.

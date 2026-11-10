---
type: SEIObservation
title: "Leitura com o cliente ureq nas sete instalações de treinamento: PASS"
description: "Ensaio de 10/11/2026 da #673/#686: login, unidade atual, recebidos e processo NUP com saída 0 na UnB, PEN1-4, UFRJ e GO, pelo cliente HTTPS sobre ureq e native-tls"
---

# Leitura com `ureq` — sete instalações de treinamento, 10/11/2026

**Regra:** aprovação para produção exige ensaio em todas as instalações de
treinamento (decisão do proprietário de 10/11/2026).
**Mudança ensaiada:** #673 / PR #686, o cliente HTTPS único passou de
HTTP/TLS/proxy escritos à mão para `ureq` 3 com `native-tls`.
**Contas:** as públicas da [tabela autorizada](../interfaces/credenciais-de-treinamento.md);
sessão nova por instalação, em pasta `0700` com arquivos `0600` fora do
repositório, apagada ao fim.
**Execução:** binário release da `main` com a #686; só leitura, nenhum
`--confirmar`. Comandos espaçados (WAF de Goiás). Saída JSON conferida
só pelo código de saída e pelo campo `codigo`.

| Comando | UnB | PEN1 | PEN2 | PEN3 | PEN4 | UFRJ | GO |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `login` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `unidade atual` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `unidade recebidos` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `processo NUP` | PASS | PASS | PASS | PASS | PASS | PASS | PASS |

PASS = saída 0. O NUP lido veio da lista de Recebidos; em Goiás, sem
Recebidos, de Gerados (`unidade gerados`, também saída 0).

O `login` exercitou o proxy do ambiente e o TLS nativo pelo `ureq`, sem
redirecionamento seguido; as leituras, o GET do Controle e as telas do
processo pelo mesmo cliente.

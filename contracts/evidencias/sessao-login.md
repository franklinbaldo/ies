---
type: OperationEvidence
title: sessao.login
operation: sessao.login
source: todos-demonstration-matrix
source_commit: 6127bd38b9182108c9b0922d6796493e338e5f05
writes: false
ies_implemented: true
upstream_steps: 0
unresolved_steps: 0
---

# sessao.login

Operação do IES, **não** um passo da matriz do Todos: toda cassette do
Todos começa com um login, mas o login não é gravado como cenário próprio
nem é reproduzível offline pelo IES (o corpo do POST leva credenciais).
A referência comportamental é o código do Todos no commit acima
(`src/todos/sei/session.py`: `_tela_de_login`, `_login_pairs`,
`_org_value`, `_aceitar_login`, `_abrir_controle_de_processos`,
`e_tela_de_login`) e os documentos `02-sessao-e-login.md` e
`sei-multi-instalacao.md` do mesmo commit.

## Observações públicas sem credencial (02/11/2026)

GET das páginas públicas de login da UnB, PEN1, UFRJ e GO, sem enviar
credenciais. As cópias ficaram privadas e **não** foram versionadas; os
testes usam telas sintéticas com a mesma estrutura.

- UnB: raiz → 302 → `sip/login.php`; botão `Acessar`.
- PEN1: raiz → 302 `/sei/` → 302 `sip/login.php` com `infra_url`;
  `sbmAcessar`, desabilitado pelo `acaoLogin(2)` servido.
- UFRJ: raiz → 302 **`http://`** `/sei/` → 302 HTTPS `/sei/` → 302
  `sip/login.php`; captcha só em CSS e JavaScript, sem controle servido.
- GO: raiz → 302 `/sei/` → 302 **`http://`** para o módulo
  `MdGoiasLoginSEI.php` (426 em HTTP; a mesma URL em HTTPS serve a tela,
  ISO-8859-1); `action` relativo ao módulo.

Nas quatro: `frmLogin`, `txtUsuario`, `pwdSenha`, `selOrgao`, `hdnAcao`
oculto e `onsubmit="return acaoLogin(2);"`. Várias respostas trazem mais de
um `Set-Cookie`.

## Limite da evidência

Nenhum login autenticado foi feito pelo IES até este card. Sucesso,
Painel de Controle, recusa de credencial, captcha exigido e 2FA vêm do
comportamento do Todos e de telas sintéticas, não de gravação do IES. A
validação live fica no roteiro do operador.

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

## Login e relogin de leitura no IES

- [Ensaio live de relogin na PEN4](../../docs/validacao-live/2026-11-05-relogin.md):
  **PASS** para login forçado, sessão desconhecida na leitura de processo,
  reaproveitamento da geração atualizada, 302 do Controle para o login
  e relogin único; a prévia de escrita não relogou (**PASS**).
- O vencimento **natural** da sessão, segunda falha em sequência e
  relogin live em outras instalações seguem **NOT_RUN** nesse ensaio.

## Limite da evidência

As observações públicas de 02/11 não incluíram autenticação. O
relatório live de 05/11 comprova apenas os cenários de login/relogin
acima na PEN4, não a cobertura universal das instalações. Recusa de
credencial, captcha exigido e 2FA seguem sustentados pelo Todos e por
testes sintéticos, sem validação live correspondente neste card.

---
type: SEIObservation
title: Sessão e login do SEI
description: SIP, módulo próprio de Goiás, redirecionamentos, formulário de login e estado autenticado
---

# Sessão e login

Na UnB 4.0.9, o login é servido pelo SIP em `/sip/login.php`.
O formulário contém usuário, senha, seleção de órgão e campos
ocultos de controle. Depois do POST, o fluxo de autenticação pode
passar por `inicializar.php` e chegar ao Controle de Processos.

A página inicial depende da instalação e do perfil. Algumas contas
começam no Painel de Controle. Siglas de órgão e de sistema, captcha,
SSO e autenticação adicional podem variar.

Os cookies autenticam a sessão. A unidade ativa é estado dessa sessão
e persiste entre comandos/requisições. Trocar de unidade altera o
contexto dos links disponíveis.

## Telas públicas de login (02/11/2026, só GET, sem credencial)

| Instalação | Cadeia a partir da raiz HTTPS | Botão |
| --- | --- | --- |
| UnB | 302 → `sip/login.php?sigla_orgao_sistema=…&sigla_sistema=SEI` | `Acessar` |
| PEN1 | 302 `/sei/` → 302 `sip/login.php?…&infra_url=…` | `sbmAcessar` |
| UFRJ | 302 **`http://`** `/sei/` → 302 HTTPS `/sei/` → 302 `sip/login.php?…` | `sbmAcessar` |
| GO | 302 `/sei/` → 302 **`http://`** `…/modulos/goias/login-sei/MdGoiasLoginSEI.php?…` | `sbmAcessar` |

- Em GO, a URL HTTP do módulo responde 426; a mesma URL em HTTPS serve
  a tela (ISO-8859-1). O `action` (`login.php?…`) é **relativo ao
  módulo**, não ao SIP.
- As quatro servem `frmLogin` `method="post"` com
  `onsubmit="return acaoLogin(2);"`, `txtUsuario`, `pwdSenha`,
  `selOrgao` (primeira opção `value="null"`) e o oculto `hdnAcao`
  (`value="1"`). `hdnInfraPrefixoCookie` fica **fora** do formulário.
- O `acaoLogin(acao)` servido grava `hdnAcao` com o argumento. Na PEN1,
  UFRJ e GO, com `acao == 2`, ele também desabilita `sbmAcessar` antes do
  envio: o navegador não manda o botão. Na UnB, `Acessar` não é
  desabilitado.
- O script também grava no navegador um cookie `SIP_U_<órgão>_SEI` com o
  órgão escolhido; o Todos loga sem ele.
- A UnB traz o órgão já `selected`; PEN1, UFRJ e GO trazem `null`
  selecionado.
- Captcha aparece só em CSS (`#txtInfraCaptcha`) e na função
  `infraGerarNovoCaptcha` nas quatro; nenhuma serve o controle
  `txtInfraCaptcha`. O mesmo vale para os estilos de 2FA. "Presente no
  HTML" não é "exigido".
- A PEN1 e GO servem um aviso de navegador `#divInfraNavegadorAviso` com
  classe `alert alert-danger`: não é recusa.
- Várias respostas trazem mais de um `Set-Cookie` (sessão PHP, afinidade
  de balanceador, cookie da instalação). Na PEN1 um deles tem `Domain` do
  domínio pai; em GO, `Domain` igual ao host com ponto inicial.

**Não generalizar:** o fato de uma instalação aceitar um envio de
login sem GET prévio não comprova que outras o farão. Não persista
credenciais, cookies ou URLs assinadas em evidências públicas. A
mensagem do SIP para senha errada não foi capturada pelo IES.

---
type: SEIObservation
title: Sessão e login do SEI
description: SIP, redirecionamentos e estado autenticado
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

**Não generalizar:** o fato de uma instalação aceitar um envio de
login sem GET prévio não comprova que outras o farão. Não persista
credenciais, cookies ou URLs assinadas em evidências públicas.

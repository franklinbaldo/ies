---
type: Guide
title: Erros de transporte e sessão
description: Bloqueio de segurança, 302 do Controle para o login e leitura coerente da sessão
---

# Erros de transporte e sessão

Parte do [guia de erros tipados](../erros-tipados.md), separada por responsabilidade.

## Bloqueio de segurança da instalação

`FalhaTransporte::BloqueioDeSegurancaDaInstalacao`
(`bloqueio_de_seguranca_da_instalacao`) é o 403 com a página de bloqueio
do WAF de GO (03/11/2026, depois de rajadas): `<title>Acesso Negado</title>`
e a frase "bloqueada por nossa política de segurança" no texto visível
do corpo; ou a do Cloudflare da PEN2 (04/11/2026, segundo GET de
`ies login`; também na PEN3, no POST da pesquisa com termo só de
dígitos): `<title>Attention Required! | Cloudflare</title>` e
"Sorry, you have been blocked". É reconhecimento de **tela** (como o da tela de login), feito
na camada de transporte para todos os comandos Web
(`src/transporte/bloqueio_de_seguranca.rs`), não leitura da mensagem para
decidir tratamento. Expõe `status_http` 403 e a mensagem saneada com o
código de bloqueio (não é segredo; URLs e cookies saem redigidos). A
sessão continuou válida: não há relogin, retry nem pausa automática. O
403 sem essa página continua `sessao_nao_autorizada`. Espace as execuções
ao vivo ([estado e instalações](../como-o-sei-funciona/estado-e-instalacoes.md)).

## 302 do Controle para o login

`FalhaTransporte::ControleRedirecionadoParaOLogin` é o 302 do GET do
Controle (`envio_post::ler_tela_via`) cujo `Location`, resolvido contra
a instalação, é uma entrada do login dela: o SIP `/sip/login.php`, o
módulo de Goiás ou a raiz `<base>/` (`src/sessao_login/entrada_do_login.rs`).
O subcódigo é o mesmo dos fluxos que já viam o 302 para o login
(`sessao_encerrada_rode_ies_login`), com `status_http` 302: o fato é o
mesmo, sessão encerrada. Outro host, outro caminho, fragmento, a recusa
de link sem assinatura, outro 3xx ou 302 sem `Location` continuam
`Redirecionamento` (`redirecionamento_recusado`). O 302 nunca é seguido
([decisão](../decisoes/sessao-login/relogin-de-leitura.md#revisão-302-do-controle-para-o-login)).

## Leitura coerente da sessão (#222)

`sessao_trocada_durante_a_leitura` (`fonte_invalida`, saída 1): os
comandos que leem a sessão sem o lock do login conferem o marcador
`<url-controle>.geracao` antes e depois de URL, origem e cookie. O
marcador mudou (ou não confere com o par lido) duas vezes seguidas: um
`ies login` ou `ies unidade trocar --confirmar` gravou no meio das duas
leituras locais. Nada foi pedido ao SEI e não há relogin; rodar de novo
lê a sessão nova. Origem divergente e cookie ilegível continuam com os
subcódigos de antes, no mesmo ponto da operação
([decisão](../decisoes/sessao-login/leitura-coerente-da-sessao.md)).

---
type: Contract
title: Transporte HTTPS e relogin
description: Um cliente HTTPS com o proxy do ambiente em todos os comandos e o relogin único nas leituras, parte do contrato da CLI
---

# Transporte HTTPS e relogin

Parte do [contrato da CLI](../cli.md), separada por responsabilidade.

## Transporte HTTPS: um cliente, proxy do ambiente em todos os comandos

Todo pedido de rede do IES (o GET do Controle, os POSTs dos comandos e o
`ies login`) passa por **um único cliente HTTPS**
(`src/transporte/cliente_https.rs`): HTTP/1.1 sobre `native-tls`, uma
conexão por pedido, `Connection: close`, `Accept-Encoding: identity`,
`User-Agent: ies/<versão>`. Antes, só o login atravessava proxy; os
demais comandos conectavam direto e falhavam em rede com saída só por
proxy.

- **Proxy:** as mesmas regras do
  [login](sessao-login.md#proxy-do-ambiente), agora para todos os
  comandos: a primeira não vazia de `https_proxy`, `HTTPS_PROXY`,
  `all_proxy`, `ALL_PROXY` (`http_proxy` não vale); exceções em
  `no_proxy`, senão `NO_PROXY` (`*`, `.sufixo`, host exato), conferidas
  antes do endereço; só `http://`, credencial apenas em
  `Proxy-Authorization: Basic` do `CONNECT` (nunca no pedido ao SEI). Um
  `CONNECT host:443` por pedido; o TLS segue com o host do SEI.
- **Falhas do proxy:** os subcódigos `endereco_de_proxy_invalido`,
  `proxy_com_esquema_nao_suportado`, `proxy_inacessivel`,
  `resposta_do_proxy_nao_interpretavel` e `proxy_recusou_o_tunel`, sem
  `status_http` (o SEI não respondeu). Nos comandos fora do login saem em
  `fonte_invalida`; o campo `status_do_proxy` continua só no `ies login`.
- **Travas dos comandos, inalteradas:** só HTTPS; origem da instalação
  conferida antes de abrir o cookie; **nenhum** redirecionamento seguido
  (3xx é `redirecionamento_recusado`, com `status_http`, corpo não lido;
  o 302 do GET do Controle para o login da instalação é
  `sessao_encerrada_rode_ies_login`; 302 do POST continua dado da operação); 30 s de tempo limite de conexão,
  leitura e escrita; HTML limitado a **8 MB** durante o recebimento
  (`Content-Length` maior é recusado antes da leitura; `chunked` e corpo
  até o fim da conexão também); corpo da recusa lido até **64 KiB**;
  `Content-Type` e charset como antes; tela de login detectada; uma única
  tentativa, sem retry.
- **Login:** mesmo cliente; lê o corpo inteiro (8 MB), guarda todos os
  `Set-Cookie` e mantém o próprio intérprete de redirecionamentos, que
  não vale para outros comandos.

Testado offline com proxy falso local (`TcpListener`) e camada TLS
substituída pela identidade: `CONNECT`, pedido no túnel, ausência de
segundo pedido, `NO_PROXY`, 3xx, limites e subcódigos. O TLS real
através do túnel não tem teste automatizado; validação live: `NOT_RUN`.

## Relogin único nas leituras (Refs #200)

`ies processo NUP`, `processo buscar`, `documento DOC`, `documento buscar`,
`unidade listar-processos`, `recebidos`, `gerados`, `atual` e `colunas` sem
`--definir` relogam no máximo uma vez, só depois de
`pagina_de_login_recebida_no_controle` ou `sessao_encerrada_rode_ies_login`,
e recomeçam do início uma vez. Antes de logar, sob o lock do `ies login`,
conferem a geração gravada: se outro processo já relogou, usam a sessão
nova sem login. A saída ganha `relogin: true` (ausente sem login deste
processo) e `requisicoes` soma a primeira tentativa, o login e a releitura.
Escrita e prévia de escrita nunca relogam. Regras, orçamento e saídas:
[relogin único nas leituras](sessao-login.md#relogin-único-nas-leituras).

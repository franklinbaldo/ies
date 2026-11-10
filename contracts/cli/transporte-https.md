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
(`src/transporte/cliente_https.rs`) sobre [`ureq`](https://crates.io/crates/ureq)
3 com `native-tls` (raízes do sistema e `SSL_CERT_FILE`),
`Accept-Encoding: identity`, `User-Agent: ies/<versão>`. Nenhum HTTP,
TLS ou proxy é escrito à mão no IES (#673).

- **Proxy:** o do ambiente, lido pelo `ureq` (`ALL_PROXY`, `HTTPS_PROXY`,
  `HTTP_PROXY`, em maiúsculas ou minúsculas), com exceções em
  `NO_PROXY`/`no_proxy`. A credencial do proxy vai só no `CONNECT`.
- **Falhas do proxy:** endereço inválido ou túnel recusado saem como
  `FalhaTransporte::Proxy` ("o proxy do ambiente é inválido ou recusou o
  túnel"), sem `status_http` (o SEI não respondeu). Demais falhas de
  conexão, TLS e tempo limite são falha de transporte, sem repetição.
- **Travas dos comandos, inalteradas:** só HTTPS; origem da instalação
  conferida antes de abrir o cookie; **nenhum** redirecionamento seguido
  (3xx é `redirecionamento_recusado`, com `status_http`, corpo não lido;
  o 302 do GET do Controle para o login da instalação é
  `sessao_encerrada_rode_ies_login`; 302 do POST continua dado da operação); 30 s de tempo limite de conexão,
  leitura e escrita; HTML limitado a **8 MB** durante o recebimento
  (`Content-Length` maior é recusado antes da leitura; o corpo sem
  tamanho declarado também); corpo da recusa lido até **64 KiB**;
  `Content-Type` e charset como antes; tela de login detectada; uma única
  tentativa, sem retry.
- **Login:** mesmo cliente; lê o corpo inteiro (8 MB), guarda todos os
  `Set-Cookie` e mantém o próprio intérprete de redirecionamentos, que
  não vale para outros comandos.

Testado offline pelos testes de integração que sobem proxy local e
servidor TLS com certificado efêmero (`SSL_CERT_FILE`): o binário faz
`CONNECT`, TLS real pelo túnel e o pedido ao SEI sintético.

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

---
type: InterfaceSource
title: SOAP oficial SEI WSDL e esquemas XML
protocol: soap-wsdl
contract_status: per-installation-dynamic
runtime_availability: unknown
document_version: 4.0-manual
---

# SEI SOAP — WSDL e XSD por instalação

O contrato formal do WebService nativo do SEI é um **WSDL**
(*Web Services Description Language*), com tipos XML Schema
(`xsd:schema`, possivelmente referências `xsd:import`/`xsd:include`).
Não confundir com o OpenAPI, que descreve o módulo REST WSSEI.

Segundo o [manual oficial de Administração do SEI](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administra%C3%A7%C3%A3o_SEI-11_Sistemas.html),
o endereço do contrato em uma instalação convencional é:

```text
https://[servidor]/sei/controlador_ws.php?servico=sei
```

O endpoint de execução de SOAP usado nas integrações convencionais é
`/sei/ws/SeiWS.php`. **Não enviar POST ao URL do WSDL**, nem presumir
que todos os servidores mantenham o mesmo `soap:address`.
É o WSDL efetivamente servido pela instalação que declara bindings,
ports, tipos, operações e o endereço real do serviço.

**O WSDL específico não está versionado neste repositório.**
Em instalações protegidas, **até obter o WSDL exige credenciais**;
não presumir acesso anônimo. Consultá-lo depende do host, da identidade,
do cadastro do sistema e da autorização para a integração.
O método de autenticação deve ser obtido do administrador, não adivinhado. Não foram
executados acessos live a SOAP nesta etapa. O manual é referência de
família de operações, não prova da disponibilidade local.

## Manuais de origem

- [Web Services SEI 4.0 (Goiás, PDF)](https://sei.goias.gov.br/manual/SEI-WebServices-v4.0.pdf)
- [Administração > Sistemas (oficial)](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administra%C3%A7%C3%A3o_SEI-11_Sistemas.html)

O manual 4.0 descreve, entre outras operações,
`consultarProcedimento`, `consultarDocumento`, `listarUnidades`,
`listarAndamentos`, `incluirDocumento`, `gerarProcedimento`
e `enviarProcesso`. Esses métodos não são automaticamente
equivalentes às leituras e escritas definidas no IES.

O serviço SOAP depende do cadastro de um sistema externo, associação
de serviços/operações, unidades e tipos permitidos. Os argumentos
incluem `SiglaSistema`, `IdentificacaoServico` e `IdUnidade`,
conforme a operação. Permissões da interface Web não são suficientes
para concluir que SOAP está autorizado.

## ISO do SOAP: codificação e versão

O exemplo de cliente do manual 4.0 (página 49) usa
`SoapClient(..., encoding='ISO-8859-1')` e apresenta
`SOAP_1_1` para o caminho opcional de anexos MTOM.

**ISO-8859-1 é a codificação do exemplo, não a identidade do
contrato.** A identidade/gramática SOAP é expressa pelo WSDL/XSD.
Não forçar ISO-8859-1 ou SOAP 1.1 sem verificar o contrato e as
respostas da instalação; não confundir o encoding XML com o charset
de respostas HTML da interface Web.

O WSDL do SIP é diferente: `/sip/controlador_ws.php?servico=sip`.
Não substitui os serviços do SEI.

## Como validar uma instalação autorizada

1. Obter do administrador o **host HTTPS**, a existência do serviço,
   a credencial necessária e a autorização para consultar o WSDL.
   Não consultar anonimamente um endpoint que exige autenticação.
2. Guardar o WSDL em local privado e identificar `wsdl:definitions`,
   `wsdl:portType`, `wsdl:binding`, `soap:address` e tipos XSD.
3. Resolver `xsd:import` e `xsd:include` somente de origens
   explicitamente autorizadas; não seguir URLs arbitrárias do XML.
4. Conferir operações autorizadas, autenticação e compatibilidade,
   sem executar operações de escrita só para testar conectividade.
5. Criar fixtures artificiais sem identificadores reais e comparar
   campos, resultados e erros com o contrato da CLI do IES.

A [política por instalação](acesso-por-instalacao.md) distingue
`auth-required` de serviço ausente. Um WSDL obtido não comprova que
cada operação SOAP tenha permissão para execução.

Conferir [REST OpenAPI](rest-wssei.md) e
[a matriz experimental](../experimentos/comparar-acessos-sei.md).

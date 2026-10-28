---
type: Guide
title: Endpoints e operações no modelo técnico do IES
description: Distinção entre contratos técnicos, interfaces implantadas e operações de domínio
---

# Endpoint não é operação de domínio

`Endpoint` é entidade **do catálogo técnico de integrações**, não
do domínio administrativo SEI. Não criar `Endpoint` dentro de
`Processo`, `Documento` ou `Andamento`.

## Conceitos

| Conceito | Identidade e responsabilidade |
| --- | --- |
| OperacaoDominio | Ação pretendida e efeito semântico, independente de transporte |
| EndpointDefinicao | Interface descrita por contrato técnico e versão desse contrato |
| ImplementacaoOperacao | Adaptador que traduz uma operação em uma ou mais chamadas e reconcilia efeitos |
| EndpointImplantado | Associação de definição a módulo e endereço de uma instalação, somente quando comprovada |
| CapacidadeOperacional | Evidência de disponibilidade, permissão e equivalência por instalação, identidade, unidade e operação |

Para REST, uma definição compreende método + padrão de rota + contrato
OpenAPI. Para SOAP, serviço/port/binding/operação e WSDL/XSD.
Para Web, ação publicada/formulário e contexto de sessão. Não
reduzir tudo a `GET /rota`: as interfaces têm identidades diferentes.
Uma operação pode requerer **múltiplos** endpoints e um endpoint pode
participar de mais de uma implementação.

## Invariantes

- Endpoint publicado na especificação não significa implantado.
- Endpoint implantado não significa autorizado à conta/unidade.
- Endpoint autorizado não comprova equivalência ao contrato do IES.
- Versão do SEI não implica versão ou presença de módulo adicional.
- `infra_hash` identifica/assina links de sessão; não é um módulo.
- Para operações com efeito, timeout ou erro não autoriza fallback
  automático de protocolo nem repetição.
- Endpoint Web pode provocar recebimento implícito ao abrir processo:
  o verbo HTTP não determina sozinho ausência de efeito administrativo.
- A mesma instalação pode mudar módulos/configurações com o tempo:
  guardar evidência com contexto e data de verificação, não booleano
  global eterno.

## Exemplo de vínculo

`processo.enviar` -> implementação REST -> definição
`POST /processo/enviar` (WSSEI) -> disponibilidade e equivalência
**não confirmadas** numa instalação específica.

A implementação Web pode envolver formulário, consulta de unidade e
confirmação posterior; não presumir uma correspondência 1:1 entre
operação e rota.

## Limite de implementação

Documentar definições no catálogo OKF. Só criar tipos Rust para
endpoints quando forem necessários a clientes REST/SOAP/Web concretos;
preferir funções por operação a um roteador genérico de endpoints.
Não armazenar URL assinada, cookie ou token no catálogo público.

Referências:
- [Endpoints REST WSSEI](../interfaces/rest-endpoints.md)
- [Arquitetura de acessos](../arquitetura/0001-rotas-por-evidencia.md)
- [Acesso por instalação](../interfaces/acesso-por-instalacao.md)
- [Modelo conceitual](modelo-conceitual-sei.md)

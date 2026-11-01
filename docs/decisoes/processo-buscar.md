---
type: ImplementationDecision
title: Pesquisa de NUP em HTML privado sem sessão
operation: processo.buscar
implementation: partial
status: adopted-for-scope
scope: seleção exata de NUP em uma resposta de pesquisa previamente capturada
selected_strategy: web-captured-offline
alternatives: web-captured-offline,web-post-authorized,rest-wssei,soap-official
evidence_level: matriz-historica-e-parser-offline
live_validation: nao-realizada
---

# Decisão de implementação de processo.buscar

## Alternativas consideradas

- **web-captured-offline:** aceita resposta HTML já obtida pelo operador
  autorizado, com parser estrito e orçamento de zero requisições.
- **web-post-authorized:** caminho futuro para pesquisa NUP fora da caixa,
  mas POST e sessão têm efeitos/permissões ainda não ensaiados no IES.
- **rest-wssei:** módulo opcional e token não comprovados por instalação.
- **soap-official:** autenticação e equivalência desconhecidas.

## Comparação

A alternativa offline evita qualquer acesso autenticado ou GET/POST, ao
custo de não comprovar a presença atual nem a cobertura da busca.
A rota Web POST pode ser mais útil, mas seus efeitos e sessão não foram
validados no IES; REST e SOAP não têm disponibilidade comprovada.

## Decisão

A primeira fatia amplia a CLI de domínio para consultar NUP em uma
resposta de pesquisa **externa à caixa** sem reutilizar links ou enviar
credenciais. O arquivo não é prova de autoria, frescura, autenticação
ou cobertura: a saída é rotulada como observação offline. Um NUP ausente
não significa inexistência no SEI. O contrato exige tabela única,
links e identificadores verificáveis, e número exato.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-buscar.md)
- [Matriz](../../contracts/evidencias/processo-buscar.md)
- [Testes de integração](../../tests/processo_pesquisa_privada.rs)
- [Testes unitários](../../src/processo_pesquisa_privada_cli/tests.rs)
- [Parser da resposta](../../src/pesquisa_web.rs)
- [Política de acesso](../interfaces/acesso-por-instalacao.md)

## Limitações e revisão

Para Web live, identificar POST real, hiddens, efeitos, autorização de
sessão e variações UnB/UFRJ/Goiás/PEN antes de implementar transporte.
Não reutilizar esta decisão como prova de autenticação, ausência ou
resultado integral; reavaliar custos e equivalência com Todos.

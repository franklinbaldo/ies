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

## Regras do `onsubmit` para a derivação offline

Fora da CLI: `derivar_pesquisa_de_protocolo` (em
`src/pesquisa_web/formulario_pesquisa.rs`) prepara o POST da
[decisão 0002](../arquitetura/0002-submeter-formulario-observado.md),
sem rede. As regras foram escritas à mão a partir do JavaScript servido no
GET de `protocolo_pesquisar` (interações 1, 3, 5 e 9) das cassettes
`processo-busca-*` de UnB, PEN1, UFRJ e Goiás. O IES não executa nem
interpreta JavaScript.

- `onSubmitForm()` exige `#optProcessos` ou `#optDocumentos` marcado. A
  tela vem com `optDocumentos` (`D`) marcado nas quatro instalações, e `D`
  exige ainda `chkSinDocumentosGerados` ou `chkSinDocumentosRecebidos`,
  checkboxes que a derivação não envia. Para buscar processo, a operação
  envia o valor servido de `#optProcessos` (nome `rdoPesquisarEm`,
  observado `P`), como o clique no rádio. Rádio ausente, repetido, sem
  `value` ou com outro nome são recusas próprias.
- Com `P`, `tratarCamposDocumento()` esvazia `hdnIdAssinante`, entre
  outros campos que a derivação não envia. Como os ocultos saem como
  servidos, `hdnIdAssinante` não vazio é recusado.
- `onSubmitForm()` exige órgão em `selOrgaoPesquisa[]` (`multipleSelect`,
  `getSelects`). A operação envia as opções com `selected` servidas, na
  ordem do documento, e não inventa órgão. Seleção servida: 1 opção na
  UnB, 2 na PEN1 e na UFRJ e 62 em Goiás. `select` ausente ou repetido,
  sem `multiple`, sem seleção ou com opção selecionada sem `value` são
  recusas próprias.
- `navegar(inicio)` só reescreve `hdnInicio` na paginação; `sbmPesquisar`
  não tem `onclick`. O valor servido foi `0` nos 16 GETs; outro valor
  pediria página posterior e é recusado.
- Contato, datas e, na UFRJ e em Goiás, faixa de valores só são conferidos
  quando preenchidos; a derivação não os envia. `onSubmitForm()` é igual
  na UnB e na PEN1, e na UFRJ e em Goiás tem essas conferências a mais. A
  operação codifica a **interseção**.

Ensaio offline: a derivação passou nos 16 formulários reais (4 GETs por
instalação), com 1, 2, 2 e 62 órgãos enviados (**PASS**, teste local
temporário com arquivos `0600` fora do repositório, não versionado).

O `select` real tem `multiple` sem valor antes de `id`, e o `tl` 0.7.8
perde o primeiro caractere do atributo seguinte a um booleano sem valor
(`id` vira `d`; `multiple disabled` vira `isabled`). Por isso o órgão é
localizado pelo `name`. O mesmo defeito pode esconder um `disabled` do
primitivo: questão aberta, sem correção nesta fatia.

As checkboxes marcadas, os selects simples e os textos vazios que o
navegador enviaria continuam fora do POST. Medir isso exige a
[coleta do formato de POST](../coleta-formato-post.md) ou ablação em
treinamento autorizado. Validação live: **NOT_RUN**.

## Limitações e revisão

Para Web live, identificar POST real, hiddens, efeitos, autorização de
sessão e variações UnB/UFRJ/Goiás/PEN antes de implementar transporte.
Não reutilizar esta decisão como prova de autenticação, ausência ou
resultado integral; reavaliar custos e equivalência com Todos.

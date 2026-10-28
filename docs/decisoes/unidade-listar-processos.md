---
type: ImplementationDecision
title: Escolha de leitura da caixa do SEI baseada em evidências
operation: unidade.listar_processos
implementation: partial
status: adopted-for-scope
scope: leitura da caixa pelo adaptador Web e consolidação offline sem paginação HTTP automática
selected_strategy: web
alternatives: web, rest-wssei, soap-official
evidence_level: offline-replay-unb-go-pen3
live_validation: not-performed
---

# Leitura da caixa da unidade — decisão limitada ao recorte demonstrado

## Alternativas consideradas

- `web`: cliente Web do SEI, interpretando a caixa HTML com Recebidos
  e Gerados, sem executar POST de paginação nem inventar URLs assinadas.
  Cassettes documentam e testam este caminho, não definem sua arquitetura.
- `rest-wssei`: avaliar `GET /processo/listar` somente se a instalação
  tiver módulo funcional e a conta/token forem autorizados; desconhecido
  neste recorte. OpenAPI público não comprova acesso ou equivalência.
- `soap-official`: avaliar WebServices configurados pela instituição;
  WSDL pode exigir credenciais para consulta e a permissão de cada
  operação precisa ser confirmada. Disponibilidade desconhecida.

## Comparação

| Alternativa | Equivalência com a caixa | Custo e fragilidade | Situação |
| --- | --- | --- | --- |
| Web | Duas tabelas, totais e primeira página testados | Parsing HTML; sessão assinada | Replay offline testado |
| REST WSSEI | Filtros, atribuição e conjuntos ainda não comparados | Resposta estruturada em princípio; instalação adicional | Não testada nas sete instalações |
| SOAP oficial | Contrato de caixa ainda não mapeado | Configuração administrativa obrigatória | Não testada |

Não há benchmark comparativo de latência, memória ou requisições REST/SOAP;
não atribuir vantagem medida a nenhum deles.

## Decisão

Adotar **`web` somente para a leitura parcial atual**: parser HTML
tipado, conferência de intervalos e replay como teste. O domínio
guarda processos e totais; `completa` é derivado na borda.
O GET live de caixa assinada ainda não foi validado com sessão real.
Não existe uma camada de observação no runtime.

A decisão não seleciona arquitetura universal do IES, não exige que
outras operações sigam o Todos e não autoriza paginação POST. O contrato
de domínio permanece independente do transporte.

## Evidências verificáveis

- [Contrato público](../../contracts/cli/unidade-listar-processos.md)
- [Matriz por instalação](../../contracts/evidencias/unidade-listar_processos.md)
- [Testes Rust de replay e integridade](../../tests/replay_unidade.rs)
- [Observação de tabelas e paginação](../como-o-sei-funciona/tabelas-e-paginacao.md)
- [Estudo de alternativas de transporte](../experimentos/comparar-acessos-sei.md)

No replay de cassettes verificadas, UnB trouxe 25 recebidos e 18
gerados; PEN3, 100/168 recebidos e 91/91 gerados; Goiás, 10/131
recebidos e 10/7939 gerados. Estes resultados sustentam o parser
offline, **não** a equivalência com REST/SOAP nem a autenticação live.

## Limitações e revisão

A revisão é obrigatória quando: houver sessão autorizada para
comparação matched-budget; REST ou SOAP estiver comprovadamente
habilitado; mudar a gramática/semântica de uma instalação; ou for
proposto login, escrita ou POST automático para a próxima página.

`completa: true` significa apenas que as posições declaradas nas páginas
lidas foram cobertas sem lacunas ou IDs duplicados, com totais
consistentes. **Não prova snapshot transacional**, nem que os
processos ficaram imutáveis durante ou após a leitura.

Antes de qualquer troca de estratégia, comparar mesmo usuário, unidade,
conjunto, filtros, autorização, cobertura, custos e casos de erro.
Se alternativas forem semanticamente distintas, não tratá-las como
substitutas. Erro ou timeout de escrita nunca provoca fallback.

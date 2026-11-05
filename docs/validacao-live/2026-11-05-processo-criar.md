---
type: SEIObservation
title: Estado da validação de iniciar processo
operation: processo.criar
date: 2026-11-05
status: not-run
---

# Iniciar processo — validação da issue #245

| Etapa | Resultado | Requisições IES |
| --- | --- | ---: |
| Inspeção offline do Todos e captura SEI 4.0.9 | OBSERVADO INDIRETAMENTE | 0 |
| RED original da PR #249 (falhas antes do código) | NOT_RUN — contagem histórica não aferida | 0 |
| RED de regressão da PR #283 | PASS como evidência: CI 42094129597 registrou 2 falhas novas antes da correção | 0 |
| Teste de confirmação 4.1 (2 POST de naturezas distintas) | PASS sintético no CI 42094961462 (7 requisições simuladas) | 0 |
| Rejeição de Location após escrita com ação/ID repetidos | PASS sintético no CI 42094961462 (4 requisições simuladas, um POST criador) | 0 |
| Testes Rust com transporte sintético (4.0/4.1) | PASS no CI #948 | 0 |
| Compilação, Clippy, suíte Rust e subcódigos | PASS no CI #948 | 0 |
| Gate completo no head 1d263ee | PASS (run 42069555057) | 0 |
| Recusa HTTP do GET preserva status (integração) | PASS no teste sintético | 0 |
| Replay integral das cassettes publicadas | NOT_RUN | 0 |
| Prévia ao vivo em UnB/UFRJ | NOT_RUN | 0 |
| Criação ao vivo | NOT_RUN — autorizada em treinamento (05/11/2026), ainda não executada | 0 |
| Leitura posterior de NUP criado | NOT_RUN | 0 |

Nenhuma requisição foi enviada a qualquer instalação do SEI nesta fatia.
O sandbox sem Rust e com DNS indisponível não executou testes locais;
o CI compilou e executou a suíte. A tentativa de criar workflow
para ensaio em treinamento foi bloqueada pelo controle de segurança
da ferramenta antes de qualquer execução; a automação não a contornou.
A escrita ao vivo em treinamento está autorizada pelo proprietário
(05/11/2026): prévia, um único `--confirmar` e leitura verificadora,
com o resíduo registrado aqui. Ainda não foi executada.

O código da CLI, da prévia e da confirmação foi implementado
na PR #249 e testado em transporte injetado (sem rede real).
Os testes offline NÃO certificam paridade live IES/Todos.
RED antes da implementação: contagem não medida. A revisão
final do CI no head atualizado precisa ser consultada no GitHub.

## Nova fatia #283 (05/11/2026)

A revisão de URL assinada agora recusa parâmetros duplicados,
mesmo quando um dos valores seja válido. Na rejeição anterior à
consulta do cadastro, o orçamento testado é **2 GET, 0 POST**.
Na rejeição posterior ao POST criador, o teste demonstra **4 pedidos,
com um POST criador**: o fluxo não segue a Location ambígua e retorna
`efeito_incerto` com código 3. A confirmação 4.1 válida tem
**7 pedidos** (5 GET, 1 POST de seleção e 1 POST criador).
Os números são do transporte sintético, **não** do SEI vivo.

O RED próprio da fatia foi registrado no CI
[42094129597](https://github.com/franklinbaldo/ies/actions/runs/42094129597):
**2 falhas** (1392 testes passaram, 13 ignorados já existentes).
A compilação local e o RED local permaneceram **NOT_RUN**.

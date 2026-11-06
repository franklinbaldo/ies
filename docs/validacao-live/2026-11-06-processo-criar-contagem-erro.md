---
type: SEIObservation
title: Contagem de requisições observadas em falhas de criação de processo
description: Evidência test-first offline e ausência de ensaio live da CLI processo criar na segunda instalação de treinamento
operation: processo.criar
date: 2026-11-06
status: not-run
---

# Issue #245 — contagem real no caminho de erro

## Observação e lacuna anterior ao RED

A CLI já declarava `requisicoes` na prévia e na criação bem-sucedida.
Após o POST falhar, o `Result::Err` descartava a contagem de chamadas
guardada somente na variável interna do fluxo. A saída pública de
`efeito_incerto` trazia causa e candidatos, mas **não trazia quantas
requisições haviam efetivamente sido tentadas**. A issue exige esse fato
mesmo quando a escrita não pode ser confirmada.

## RED sintético antes do código

Commit `aed3038`: três testes ativos de transporte com contador
deliberadamente ainda não ligado. CI
[42305129633](https://github.com/franklinbaldo/ies/actions/runs/42305129633),
job `prek`: **3 FAIL novos / 1.773 PASS / 22 ignored existentes**.
Todos os testes novos falharam observando `contador=0` na implementação
incompleta, embora a rede simulada tivesse sido chamada; não houve
enfraquecimento ou `#[ignore]` novo.

## GREEN / desenho e orçamentos offline

O fluxo único recebe contador mutável de requisições da CLI e o
incrementa **imediatamente antes** de cada GET ou POST: não adiciona
requisições nem modifica as regras de efeito incerto. Em erro, JSON e
apresentação humana publicam o número real, inclusive recusa antes da
escrita, POST de resposta incerta e falha na releitura. Separação de
leitura de URLs observadas em `src/processo_criar/links.rs` mantém cada
arquivo abaixo do limite de tamanho sem mudar navegação ou fronteiras.

| Cenário sintético | Chamadas tentadas | POST criador | Resultado |
| --- | ---: | ---: | --- |
| Unidade divergente após abrir Controle | 1 | 0 | erro antes da escrita |
| Prévia sem `--confirmar` | 3 | 0 | prévia sem escrita |
| POST sem resposta com candidato único conferido | 8 | 1 | saída 3, possível resíduo, sem prova causal |

Nenhuma requisição a mais, nenhum retry, nenhum JS executado, sem
cookies ou URLs assinadas em fixtures/diagnóstico. O contador é evidência
de atividade de transporte, **não comprovação de efeito**. A prova de
GREEN é o CI no head final da PR #340; não extrapolar o RED a PASS.

## Ensaio live e resíduos

**Prévia autenticada em segunda instalação de TREINAMENTO: NOT_RUN.**
**Único `--confirmar`: NOT_RUN. Verificação independente: NOT_RUN.**
**Total real nesta execução: 0 requisições ao SEI, 0 POST,
0 escritas e nenhum resíduo novo.**

Este sandbox não tem Rust/Cargo nem DNS para o acesso necessário, tampouco
sessão autenticada de treinamento com percurso completo garantido.
Não contornei esses limites nem acessei produção. O processo histórico
da UnB treinamento `23106.001875/2026-93` (ID 51731) já foi criado
em execução anterior: **não repetir o POST**. A issue permanece WIP até
validar prévia → confirmação única → leitura e registrar o resíduo numa
outra instalação segura.

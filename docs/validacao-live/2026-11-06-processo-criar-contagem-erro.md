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

## Continuação — maxlength publicado no cadastro (#245 / Todos #1248)

**Observação anterior ao RED:** a captura 4.0.9
`tests/paginas_reais/procedimento_gerar.html` do Todos @`323eb01`
mostra o `input txtDescricao` com `maxlength="100"`; captura GO 5.0.3
também publica esse atributo. Não foi necessário acessar produção.

**RED test-first:** branch da PR #340, commit `b57fde8` com quatro
cenários de transporte sintético (três negativos, um positivo).
[CI 42311417257](https://github.com/franklinbaldo/ies/actions/runs/42311417257):
**3 FAIL novos / 1.777 PASS / 22 ignored preexistentes**.
Antes da implementação a prévia de 101 ASCII retornava sucesso,
a confirmação chegava a um POST simulado e 51 emoji não-BMP não
eram recusados. Rustfmt do teste test-first também ficou pendente;
corrigir sem enfraquecer testes.

**Implementação e orçamento sintético esperado:** interpretar somente o
`maxlength` servido na tela da sessão, contar comprimento UTF-16,
recusar `101 > 100` tanto na prévia quanto na confirmação, e
`102 > 100` para 51 emoji, todos com **3 GET / 0 POST criador**.
Para 100 caracteres ou formulário sem limite, a prévia continua
sem escrita (3 GET). Atributo ilegível é erro de HTML, não
permissão para escrever. Nenhuma requisição adicional foi introduzida.

**Live desta continuação:** prévia em segunda instalação de treinamento
**NOT_RUN (0 pedidos reais)**; confirmação **NOT_RUN (0 POST real)**;
verificação **NOT_RUN (0 GET real)**. Total **0 requisições
reais / 0 escritas / 0 novos resíduos**. O NUP histórico da UnB de
treinamento não deve receber novo POST; nenhuma instalação de produção
foi acessada. A issue permanece aberta até o ensaio autenticado
completo numa segunda instalação.

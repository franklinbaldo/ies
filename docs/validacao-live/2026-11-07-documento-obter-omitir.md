---
type: SEIObservation
title: Leitura de documento com omissão explícita — ensaio não executado
description: Validação da opção --ocultar-inacessiveis da CLI do IES na issue 60 em 07/11/2026
---

# Documento obter com `--ocultar-inacessiveis`

**BDD offline:** RED na suíte Rust do CI antes da implementação; testes
de roteamento do executável e agregação tipada com contagem das
requisições simuladas. O GREEN exige o CI do head exato.

**LIVE treinamento: NOT_RUN.** Sem sessão autenticada de treinamento e
sem DNS no sandbox. Zero requisições reais ao SEI (0 GET, 0 POST),
zero escritas e zero resíduos novos. O caso da UnB com `.src` vazio
está na [observação de 04/11](2026-11-04-documento-obter.md); a
instalação não foi reconsultada.

**Próximo ensaio:** ler um documento acessível e outro cuja árvore
comprovadamente não publica `.src`, em uma instalação de treinamento.
Conferir contagem HTTP observada, nenhum retry e nenhum dado do
documento omitido na saída.

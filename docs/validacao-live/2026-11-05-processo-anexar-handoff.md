---
type: SEIObservation
title: Validação de processo anexar em 5 de novembro de 2026
description: Evidência e lacunas da fatia de prévia da issue 270
---

# Validação live — processo anexar (#270)

**Status: NOT_RUN.** Instalações UnB e segunda instalação de treinamento: **0 requisições** executadas; sem leitura ou escrita ao vivo nesta sessão. **Resíduo no SEI: nenhum**. Nenhuma instalação de produção foi acessada.

**Observação antes do RED:** fonte pública `Todos/src/todos/anexacao.py` informa UnB 4.0.9, 15/10/2026, e a sequência tela → duas consultas AJAX → POST único → 302 → releitura. Os oito cenários sintéticos Rust foram derivados dessa fonte; não são replay de cassette. O RED no CI do commit `a667d436` teve **8 falhas** (1360 passaram, 13 ignorados) na execução 42084762362. A implementação subsequente separa leitura/decisão de escrita.

**Pendente:** rodar replay das cassettes de anexação, prévia em UnB e outra instalação, testar transporte injetado e somente então habilitar escrita por um `--confirmar` após prévia ao vivo e conferir por leitura. Sem essa evidência não há PASS.

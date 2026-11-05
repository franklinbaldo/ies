---
type: Guide
title: Validação da criação de documento interno (issue 246)
description: Resultado das verificações e dos passos ainda não executados
---

# Documento criar — 05/11/2026

Operação: `documento.criar`, issue
[246](https://github.com/franklinbaldo/ies/issues/246).

| Ensaio | Resultado | Requisições |
| --- | --- | --- |
| Leitura das fontes do Todos e das capturas | PASS documental | 0 |
| Testes sintéticos do núcleo Rust | Dependente do CI da PR #248 | 0 |
| Replay integral das cassettes públicas | NOT_RUN | 0 |
| Prévia Web em treinamento | NOT_RUN | 0 |
| Criação ao vivo no SEI | NOT_RUN — sem autorização na issue | 0 |

## Alcance

O branch da PR #248 integra o subcomando e uma prévia Web estritamente de leitura: exige processo aberto na unidade, link assinado e único de inclusão, tipo exato e formulário servido compatível. Na variante da lista cujo link exige script, a navegação recusa tipadamente. O `--confirmar` recusa antes de ler a sessão. Novos testes sintéticos de CLI, links e cadastro foram publicados; **RED local não medido**, pois o sandbox não dispõe de Rust e não resolve o domínio do repositório para instalação. O último CI anterior à alteração não prova o novo head. O CI atual ainda precisa ser confirmado. Prévia live: NOT_RUN (0 requisições); escrita live: NOT_RUN (sem autorização).

## Para retomar

Implementar o fluxo Web completo com os links servidos, cadastro e
formulário avaliados, e depois testar em treinamento obedecendo ao
orçamento e intervalos da issue. A escrita em treinamento exige a
frase literal **escrita autorizada** escrita pelo proprietário na
issue #246; sem isso, permanece NOT_RUN.

## Handoff WIP — sessão de 05/11

- PR Draft #248, branch `codex/246-documento-criar`: CLI, extração do link de inclusão, prévia e leitores tipados publicados em commits da própria PR.
- Testes novos: **3 cenários CLI**, **2 de cadastro/seleção**, **2 de barra assinada** (mais os existentes). Testes adicionados antes do código, porém **RED local NOT_RUN**: não há `cargo`/`rustc` e o sandbox não resolve `github.com`.
- **CI do novo head: NOT_RUN** até a última consulta. O último CI verde é do head antigo `be643696`, não valida esta fatia. Sem execução local, a compilação e os gates do novo código **não estão comprovados**.
- **Live prévia: NOT_RUN**, 0 requisições enviadas ao SEI nesta execução. **Live escrita: NOT_RUN**, 0 escritas, sem autorização.
- **Pendente**: obter GREEN (rustfmt/Clippy/testes/OKF/subcódigos), replay do acervo, ensaio de prévia em treinamento, navegação para instalações que só publicam `escolher(id)`, campos obrigatórios/hipóteses legais, POST único de criação e releitura de confirmação. Nenhuma dessas etapas deve ser considerada entregue.
- **Próxima sessão**: executar gates do branch e corrigir falhas; manter a confirmação desabilitada até derivação, testes e autorização ao vivo. Não repetir POST nem acessar produção.

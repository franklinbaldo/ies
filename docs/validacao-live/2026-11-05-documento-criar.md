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
| Criação ao vivo no SEI | NOT_RUN — autorizado somente em treinamento, sem execução | 0 |

## Alcance

O branch da PR #248 integra o subcomando e uma prévia Web estritamente de leitura: exige processo aberto na unidade, link assinado e único de inclusão, tipo exato e formulário servido compatível. Na variante da lista cujo link exige `escolher(id)`, o novo fluxo exige prova literal da função publicada e deriva um POST somente de navegação, validando o 302 e lendo o cadastro. Se não puder comprovar a função, recusa tipadamente. O branch agora contém caminho WIP para `--confirmar`: deriva um POST do formulário servido e tenta comprovar exatamente um documento novo na árvore. **Não está validado por compilação, CI ou replay**, portanto não é entrega pronta para integrar. Novos testes sintéticos de CLI, links e cadastro foram publicados; **RED local não medido**, pois o sandbox não dispõe de Rust e não resolve o domínio do repositório para instalação. O último CI anterior à alteração não prova o novo head. O CI atual ainda precisa ser confirmado. Prévia live: NOT_RUN (0 requisições); escrita live: NOT_RUN (autorização presente, sem execução).

## Para retomar

Implementar o fluxo Web completo com os links servidos, cadastro e
formulário avaliados, e depois testar em treinamento obedecendo ao
orçamento e intervalos da issue. A escrita em treinamento **foi autorizada pelo proprietário em 05/11/2026**, mas permanece NOT_RUN até haver gates verdes, prévia ao vivo, um único `--confirmar` e verificação por leitura. O resíduo precisa ser registrado aqui.

## Handoff WIP — sessão de 05/11

- PR Draft #248, branch `codex/246-documento-criar`: CLI, extração do link de inclusão, prévia e leitores tipados publicados em commits da própria PR.
- Testes novos: **3 cenários CLI**, **2 de cadastro/seleção**, **2 de barra assinada** (mais os existentes). Testes adicionados antes do código, porém **RED local NOT_RUN**: não há `cargo`/`rustc` e o sandbox não resolve `github.com`.
- **CI do novo head: NOT_RUN** até a última consulta. O último CI verde é do head antigo `be643696`, não valida esta fatia. Sem execução local, a compilação e os gates do novo código **não estão comprovados**.
- **Live prévia: NOT_RUN**, 0 requisições enviadas ao SEI nesta execução. **Live escrita: NOT_RUN**, 0 escritas nesta sessão, com autorização apenas para treinamento.
- **Pendente**: obter GREEN (rustfmt/Clippy/testes/OKF/subcódigos), replay do acervo, ensaio de prévia em treinamento, navegação para instalações que só publicam `escolher(id)`, campos obrigatórios/hipóteses legais, POST único de criação e releitura de confirmação. Nenhuma dessas etapas deve ser considerada entregue.
- **Próxima sessão**: executar gates do branch e corrigir falhas; não executar a confirmação ao vivo antes de compilar e testar o fluxo WIP, revisar o orçamento e fazer prévia no treinamento. Não repetir POST nem acessar produção.

## Avanço WIP — reconciliação e POST derivado

- A reconciliação agora pertence ao binário de produção (antes era somente teste), recusa IDs repetidos/vazios nos retratos inicial e final, exige preservação dos anteriores e um único novo documento com número SEI.
- Testes sintéticos novos para snapshots ambíguos e derivação do formulário Web: campos ocultos, operação de escrita, URL com hash único e rejeição de campo repetido. RED **NOT_RUN** (0 falhas observadas, contagem não medida).
- A CLI já tem caminho para confirmação somente na tela 4.0 com link direto, nível público e regra de submissão observada. A implementação deriva um único POST, nunca o repete, e exige releitura antes de declarar sucesso. Falha posterior vira saída 3. **Ainda não compilado ou testado neste sandbox**; não usar como comprovação de segurança.
- Prévia em instalação de treinamento: **NOT_RUN**, 0 requisições desta sessão. Confirmação: **NOT_RUN**, 0 escritas; verificação ao vivo **NOT_RUN**, nenhum resíduo criado ou observado.
- Próximo passo: corrigir eventuais gates, executar replay das cassettes, ampliar navegação `escolher(id)`, validar campos obrigatórios/hipóteses legais e fazer prévia ao vivo antes de qualquer escrita.

### Handoff da sessão atual

- Novo RED publicado antes do GREEN no branch: 2 testes de unicidade de snapshots, 1 de valor de rádio inesperado e 3 de formulário; mais um caso de nível explícito. **RED local NOT_RUN; falhas observadas: indeterminadas**, pois não há Rust/Cargo e a resolução DNS para instalação falhou.
- O fluxo de confirmação está implementado em WIP e usa o mesmo branch da PR #248, **sem prova de compilação ou CI do head**. Não considerar pronto para merge/revisão final; confirmar gates antes de qualquer teste ao vivo.
- Prévia, POST de escrita e verificação live no SEI: **NOT_RUN, 0 requisições, 0 documentos criados**, sem resíduo desta sessão. Autorização restrita a treinamento permanece válida.
- Pendências: compilação, rustfmt/Clippy/testes/OKF/subcódigos; replay; teste de integração com orçamento exato; validação dos campos obrigatórios e hipóteses legais; verificação ao vivo da navegação `escolher(id)`; prévia e escrita única em treinamento com releitura e resíduo registrado.

---
type: CommandContract
title: Relacionar e desfazer relações de processos
operation: processo.relacionar
command: ies processo relacionar NUP --com NUP [--remover] [--confirmar]
effect: write-process-state
implementation: partial
---

# Relacionar processos (#258)

**Prévia disponível; confirmação E2E somente em TLS local sintético.**
Na CLI de distribuição, `--confirmar` para adição e remoção continua recusado
após a leitura local da sessão e **antes de qualquer requisição**.
Para provar BDD-258-02/03/04 sem escrita externa, somente a compilação
**debug** com origem exata `https://sei.test/sei` (domínio reservado)
exercita a confirmação contra anti-IES/SQLite via proxy TLS loopback.
Essa exceção de teste não libera produção ou treinamento. A prévia sem
`--confirmar` permanece disponível.
Uma relação já existente é `sem_efeito` sem escrita. O aceite do POST
mutador deve ser explícito e a leitura independente precisa comprovar
o estado pretendido; caso contrário, saída 1, sem repetição.

Abertura com a unidade da sessão exigida: a barra do nó PROCESSO fornece
procedimento_relacionar como URL assinada. GET da tela de relacionamentos,
um formulário frmProcedimentoListarRelacionamentos e uma tabela com summary
Lista de Processos Relacionados. Só os links das linhas da tabela comprovam
vínculos. Uma linha repetida ou tabela repetida é recusa. Na PEN1, a lista
vazia pode vir sem tabela: nesse caso, apenas o formulário único com
`txtProtocolo` e `hdnIdProtocolo` únicos permite a prévia; não prova
qualquer vínculo existente.

**Prévia de adicionar:** a consulta AJAX protocolo_RI1023 declarada na tela
recebe idProcedimento com o destino informado. O XML deve publicar
IdProcedimento decimal e ProtocoloProcedimentoFormatado, ambos únicos.
Não relaciona consigo mesmo. Se a própria tabela já trouxer o NUP
pedido em uma única linha, o IES publica o id e `sem_efeito: true`
diretamente: **abertura + 1 GET da tela, zero POST**, sem pedir o
complementar AJAX. Duas linhas com o mesmo NUP são ambíguas e recusadas
sem POST. Só para destino ainda não relacionado, a consulta AJAX
identifica o id: **abertura + 1 GET da tela + 1 POST de consulta
(leitura lógica)**. Quando a resposta publica `NomeTipoProcedimento`,
a prévia também mostra o tipo do processo de destino (modo humano e
campo JSON `tipo_destino`); sua ausência é `null`, não ausência de tipo.
A resposta complementar é XML (application/xml
ou text/xml), não HTML; o IES verifica o id e o NUP retornados.
A sequência completa de letras e dígitos do NUP identificado precisa
corresponder à solicitada, ignorando caixa e pontuação. Um destino
divergente é recusado antes de qualquer escrita.

Ao localizar um vínculo já publicado, inclusive para remover, o NUP
informado pode vir sem pontuação: primeiro prevalece texto exato sem
distinguir caixa; na ausência dele, comparam-se letras e dígitos
na ordem, como em Todos @323eb01. Dois equivalentes sem texto exato
são ambíguos e recusados. Uma solicitação para relacionar o próprio
processo também é recusada antes do AJAX. Protocolos alfanuméricos
não são reduzidos a seus dígitos. Nenhum desses casos escreve
relacionamento.

**Prévia de remover:** encontra linha pelo NUP informado e exige
acaoRemover('id'); na mesma linha. Sem relacionamento, sem_efeito true.
Orçamento: abertura + 1 GET da tela, sem POST.

**Adição (motor WIP, E2E somente em debug com origem .test):** o IES submete o formulário observado via botão
`sbmAdicionar`, com `txtProtocolo`, `txtIdentificacaoProtocolo` e
`hdnIdProtocolo` obtidos do XML de consulta. Um único POST mutador
é seguido de uma GET independente, que deve localizar exatamente o id
e o NUP de destino na tabela. Aceita somente 302 de retorno à ação
`procedimento_relacionar`, com `resultado=1`; um HTTP 200 genérico,
403, transporte falho, 302 divergente ou vínculo ausente na releitura
resulta em `nao_confirmado`, saída 1. Orçamento sintético a frio:
**4 GET + 1 POST de consulta + 1 POST modificador + 1 GET**.
O sinal 302 e os campos usados provêm da tela PEN1 e do oráculo Todos,
sem alegação de validação ao vivo nas demais instalações.

**Remoção (motor WIP, E2E somente em debug com origem .test):** um único POST com `hdnInfraItemId` do vínculo
identificado, action `procedimento_excluir_relacionamento` extraída do
`acaoRemover(id)` ativo e conferida pelo primitivo de formulário. O POST
precisa devolver 302 sem seguir o redirecionamento, para a própria tela
`procedimento_relacionar` com `resultado=1`; então uma GET independente
deve constatar a ausência do vínculo. Orçamento sintético a frio:
**4 GET + 1 POST mutador + 1 GET de verificação**. Se o envio falhar,
retornar 200 genérico, 302 inválido ou a releitura não comprovar a remoção,
o efeito é **incerto, saída 1, sem retry**, preservando o status HTTP quando
conhecido. Mesmo a ausência da linha depois de falha do POST não comprova
o nosso envio (possível alteração concorrente).

Campos de sucesso JSON: `nup`, `id_procedimento`, `destino`, `id_destino`
(null quando não observado), `tipo_destino` (null quando ausente),
`remover`, `confirmado`, `alterado`, `sem_efeito`, `origem: web` e
`requisicoes`. Os três indicadores decorrem de um único estado de domínio:
prévia, sem efeito ou confirmação conferida. Nunca expor URL, `infra_hash`,
cookie, senha ou argumento secreto. Saídas: 0 prévia/sem efeito/confirmado,
1 recusa pré-envio, 2 uso inválido, 3 escrita não confirmada após envio.

## Evidência

- Todos @323eb01: src/todos/processos.py::relacionar,
  tests/test_relacionar_010.py, tests/test_processo_relacionar_remover_010.py,
  tests/paginas_reais/procedimento_relacionar.html.
- Cassette PEN1 processo-relacionar-remover-pen1: 25 interações agregadas;
  replay completo IES NOT_RUN.
- Testes sintéticos src/processo_relacionar/tests.rs e tests_fluxo.rs.
- Handler de remoção: função `acaoRemover(id)` no script ativo, com campo,
  action e submit literais; não se infere a rota de comentário, template
  ou literal solto. A derivação e o POST único são testados offline em
  `src/processo_relacionar/tests_escrita.rs`.
- RED em CI da PR #390; GREEN exige CI no SHA que inclui adição.
  A adição e a remoção foram exercitadas com respostas sintéticas,
  não na instalação. A **prévia PEN1 após a correção passou** em
  07/11/2026, com `exit 0`, `requisicoes: 4`, `confirmado: false` e
  `alterado: false` ([evidência #258](https://github.com/franklinbaldo/ies/issues/258#issuecomment-6712482131)).
  UnB NOT_RUN; escrita real NOT_RUN, nenhum retry nem resíduo.

## Subcódigos tipados da fatia

| Subcódigo | Condição observável ou regra |
| --- | --- |
| `relacionar_nao_publicado_na_barra` | A barra do processo não publica `procedimento_relacionar` |
| `links_de_relacionar_repetidos_na_barra` | Mais de um link de relacionar na barra do nó PROCESSO |
| `endereco_de_relacionar_nao_reconhecido` | O link da barra não tem a forma observada de `procedimento_relacionar` |
| `endereco_de_relacionar_de_outro_processo` | O link da barra aponta para outro `id_procedimento` |
| `tela_de_relacionamentos_redirecionada` | O GET da tela de relacionamentos respondeu com redirecionamento |
| `relacionamento_consigo_mesmo` | O destino informado é o próprio processo |
| `remocao_de_relacionamento_nao_oferecida` | A linha do relacionamento existe, mas não traz `acaoRemover('id')` |
| `destino_da_consulta_diverge_do_pedido` | O XML identificou protocolo que não equivale ao pedido completo (letras e números) |
| `resposta_da_consulta_nao_xml` | A consulta retornou resposta HTTP 200 com mídia diferente de XML |
| `acao_excluir_relacionamento_nao_declarada` | Handler `acaoRemover` não prova action única com campo e submit |
| `id_da_origem_na_action_ausente` | Link de origem não oferece id para conferir a action |
| `action_de_remocao_de_outro_processo` | Script de remoção aponta para outro processo |
| `nao_confirmado` | POST mutador feito ou aceito, mas não comprovou causalmente o efeito; saída 1 |

Outros subcódigos já definidos pelo transporte, abertura e primitivo de
formulário conservam sua identidade. Recusas não autorizam retry.

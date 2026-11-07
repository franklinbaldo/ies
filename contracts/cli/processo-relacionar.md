---
type: CommandContract
title: Relacionar e desfazer relações de processos
operation: processo.relacionar
command: ies processo relacionar NUP --com NUP [--remover] [--confirmar]
effect: write-process-state
implementation: partial
---

# Relacionar processos (#258)

**Nesta PR, somente prévia Web está implementada.** O --confirmar
recusa antes de abrir a sessão ou enviar qualquer requisição, com subcódigo
confirmacao_de_relacionamento_nao_validada.

Abertura com a unidade da sessão exigida: a barra do nó PROCESSO fornece
procedimento_relacionar como URL assinada. GET da tela de relacionamentos,
um formulário frmProcedimentoListarRelacionamentos e uma tabela com summary
Lista de Processos Relacionados. Só os links das linhas da tabela comprovam
vínculos. Uma linha repetida ou uma tabela ausente/repetida é recusa.

**Prévia de adicionar:** a consulta AJAX protocolo_RI1023 declarada na tela
recebe idProcedimento com o destino informado. O XML deve publicar
IdProcedimento decimal e ProtocoloProcedimentoFormatado, ambos únicos.
Não relaciona consigo mesmo. Se a própria tabela já trouxer o NUP
pedido em uma única linha, o IES publica o id e `sem_efeito: true`
diretamente: **abertura + 1 GET da tela, zero POST**, sem pedir o
complementar AJAX. Duas linhas com o mesmo NUP são ambíguas e recusadas
sem POST. Só para destino ainda não relacionado, a consulta AJAX
identifica o id: **abertura + 1 GET da tela + 1 POST de consulta
(leitura lógica)**. Nenhum desses casos escreve relacionamento.

**Prévia de remover:** encontra linha pelo NUP informado e exige
acaoRemover('id'); na mesma linha. Sem relacionamento, sem_efeito true.
Orçamento: abertura + 1 GET da tela, sem POST.

A escrita futura requer um POST derivado da tela servida, uma única vez,
sem seguir 302 e com releitura independente da mesma lista. Efeito incerto
sai 3. A remoção requer a action procedimento_excluir_relacionamento
observada dentro de acaoRemover, nunca texto solto. Não é implementada
enquanto faltarem provas de escrita única e de verificação.

Campos de sucesso JSON: nup, id_procedimento, destino, id_destino (null
quando não observado), remover, confirmado false, alterado false,
sem_efeito, origem web e requisicoes. Nunca expor URL, infra_hash, cookie,
senha ou argumento secreto. Exit 0 prévia, 1 recusa, 2 uso inválido.

## Evidência

- Todos @323eb01: src/todos/processos.py::relacionar,
  tests/test_relacionar_010.py, tests/test_processo_relacionar_remover_010.py,
  tests/paginas_reais/procedimento_relacionar.html.
- Cassette PEN1 processo-relacionar-remover-pen1: 25 interações agregadas;
  replay completo IES NOT_RUN.
- Testes sintéticos src/processo_relacionar/tests.rs.
- RED local NOT_RUN; CI a verificar. Live PEN1/UnB NOT_RUN (0 pedidos);
  escrita NOT_RUN, resíduo nenhum.

## Subcódigos tipados da fatia

| Subcódigo | Condição observável ou regra |
| --- | --- |
| `confirmacao_de_relacionamento_nao_validada` | `--confirmar` nesta fatia: saída 1 antes de abrir sessão ou rede |
| `relacionar_nao_publicado_na_barra` | A barra do processo não publica `procedimento_relacionar` |
| `links_de_relacionar_repetidos_na_barra` | Mais de um link de relacionar na barra do nó PROCESSO |
| `endereco_de_relacionar_nao_reconhecido` | O link da barra não tem a forma observada de `procedimento_relacionar` |
| `endereco_de_relacionar_de_outro_processo` | O link da barra aponta para outro `id_procedimento` |
| `tela_de_relacionamentos_redirecionada` | O GET da tela de relacionamentos respondeu com redirecionamento |
| `relacionamento_consigo_mesmo` | O destino informado é o próprio processo |
| `remocao_de_relacionamento_nao_oferecida` | A linha do relacionamento existe, mas não traz `acaoRemover('id')` |

Outros subcódigos já definidos pelo transporte, abertura e primitivo de
formulário conservam sua identidade. Recusas não autorizam retry.

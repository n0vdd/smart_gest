//TODO automatizar criacao de nota fiscal de servico
//pelo site da prefeitura de nova lima
//talvez tenha como enviar por xml(mais dificil)
//? devo precisar de alguma crate para xml(tenho alguns docs sobre)
//talvez precisa auatomatizar como se fosse uma pessoa(eles nao devem verificar bots, so nao posso derrubar o site)

//enviar um email com a nota gerada apos pagamento

use crate::handlers::clients::Cliente;


//TODO gera nota fiscal para os clientes que tiverem o pagamento confirmado
//TODO pegar os valores para o scraper usando f12
pub async fn gera_nfs(cliente:Cliente) {
    //TODO login no sistema da prefeitura de nova lima

    //TODO clicar no link de gerar nota fiscal no canto direito 

    //TODO preencher os campos com os dados necessarios(alguns podem ser hardcoded?)
    //? talvez tenha que refazer a estrutura de dados dos planos para incluir as coisas fiscais
    //ai pego tudo pelo cliente(plano esta relacionado ao cliente entao fica facil)

    //enviar e rececer a nota fiscal
    //salvar a mesma para o sistema de arquivos
    //caminho: notas_fiscais/{cliente_nome}/{data}.pdf
    //e salvar um o pagamento relacionado,o caminho e a data no banco de dados
}
